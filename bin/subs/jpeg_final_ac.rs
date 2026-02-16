use anyhow::{anyhow, bail, Context, Result};
use image::ImageFormat;
use sciimg::prelude::{Image, ImageMode};
use std::collections::HashMap;
use std::path::Path;

pub fn load_jpeg_with_zeroed_final_ac(path: &Path) -> Result<Image> {
    let original = std::fs::read(path)
        .with_context(|| format!("Failed to read input JPEG: {}", path.display()))?;

    let modified = zero_final_ac_in_jpeg_bytes(&original).with_context(|| {
        format!(
            "Failed to apply --zero-final-ac transform to {}",
            path.display()
        )
    })?;

    let decoded = image::load_from_memory_with_format(&modified, ImageFormat::Jpeg)
        .with_context(|| format!("Failed to decode transformed JPEG: {}", path.display()))?;

    dynamic_to_sciimg_image(decoded)
}

fn dynamic_to_sciimg_image(dynamic: image::DynamicImage) -> Result<Image> {
    let rgb = dynamic.into_rgb8();
    let (width, height) = rgb.dimensions();

    let mut out = Image::new_with_bands(width as usize, height as usize, 3, ImageMode::U8BIT)?;

    for (x, y, pixel) in rgb.enumerate_pixels() {
        out.put(x as usize, y as usize, pixel[0] as f32, 0);
        out.put(x as usize, y as usize, pixel[1] as f32, 1);
        out.put(x as usize, y as usize, pixel[2] as f32, 2);
    }

    Ok(out)
}

#[derive(Clone, Debug)]
struct FrameComponent {
    identifier: u8,
    h_samp: u8,
    v_samp: u8,
}

#[derive(Clone, Debug)]
struct FrameInfo {
    width: u16,
    height: u16,
    components: Vec<FrameComponent>,
    max_h_samp: u8,
    max_v_samp: u8,
}

#[derive(Clone, Debug)]
struct ScanComponent {
    dc_table: usize,
    ac_table: usize,
    h_samp: u8,
    v_samp: u8,
}

#[derive(Clone, Debug)]
struct ScanInfo {
    components: Vec<ScanComponent>,
}

#[derive(Clone, Debug)]
struct HuffmanCoding {
    decode: HashMap<(u8, u16), u8>,
    encode: [Option<(u16, u8)>; 256],
}

fn zero_final_ac_in_jpeg_bytes(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        bail!("Input is not a valid JPEG (missing SOI marker)");
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[..2]);

    let mut index = 2;
    let mut frame: Option<FrameInfo> = None;
    let mut dc_tables: [Option<HuffmanCoding>; 4] = std::array::from_fn(|_| None);
    let mut ac_tables: [Option<HuffmanCoding>; 4] = std::array::from_fn(|_| None);
    let mut restart_interval: usize = 0;

    while index < data.len() {
        let (segment_start, marker_index, marker) = next_marker(data, index)?;

        if marker == 0xD9 {
            out.extend_from_slice(&data[segment_start..=marker_index]);
            return Ok(out);
        }

        if marker == 0xDA {
            let frame = frame
                .as_ref()
                .ok_or_else(|| anyhow!("SOS encountered before SOF marker"))?;

            let segment_end = segment_end_with_length(data, marker_index)?;
            let scan_payload = &data[(marker_index + 3)..segment_end];
            let scan = parse_sos(scan_payload, frame)?;

            out.extend_from_slice(&data[segment_start..segment_end]);

            let entropy = &data[segment_end..];
            let (rewritten_entropy, marker_offset) = rewrite_scan_entropy(
                entropy,
                frame,
                &scan,
                &dc_tables,
                &ac_tables,
                restart_interval,
            )?;

            out.extend_from_slice(&rewritten_entropy);
            out.extend_from_slice(&entropy[marker_offset..]);
            return Ok(out);
        }

        if is_sof_progressive_or_unsupported(marker) {
            bail!(
                "Unsupported JPEG coding process for --zero-final-ac (marker 0xFF{:02X})",
                marker
            );
        }

        let segment_end = if marker_has_length(marker) {
            segment_end_with_length(data, marker_index)?
        } else {
            marker_index + 1
        };

        if marker == 0xC0 || marker == 0xC1 {
            let payload = &data[(marker_index + 3)..segment_end];
            frame = Some(parse_sof(payload)?);
        } else if marker == 0xC4 {
            let payload = &data[(marker_index + 3)..segment_end];
            parse_dht(payload, &mut dc_tables, &mut ac_tables)?;
        } else if marker == 0xDD {
            let payload = &data[(marker_index + 3)..segment_end];
            restart_interval = parse_dri(payload)?;
        }

        out.extend_from_slice(&data[segment_start..segment_end]);
        index = segment_end;
    }

    bail!("Unexpected end of JPEG while searching for SOS")
}

fn next_marker(data: &[u8], index: usize) -> Result<(usize, usize, u8)> {
    if index >= data.len() {
        bail!("Unexpected end of JPEG while parsing marker");
    }

    if data[index] != 0xFF {
        bail!("Invalid marker prefix at byte {}", index);
    }

    let segment_start = index;
    let mut marker_index = index + 1;

    while marker_index < data.len() && data[marker_index] == 0xFF {
        marker_index += 1;
    }

    if marker_index >= data.len() {
        bail!("Unexpected end of JPEG while reading marker code");
    }

    Ok((segment_start, marker_index, data[marker_index]))
}

fn marker_has_length(marker: u8) -> bool {
    !matches!(marker, 0x01 | 0xD0..=0xD9)
}

fn is_sof_progressive_or_unsupported(marker: u8) -> bool {
    matches!(
        marker,
        0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE | 0xCF
    )
}

fn segment_end_with_length(data: &[u8], marker_index: usize) -> Result<usize> {
    if marker_index + 2 >= data.len() {
        bail!("Truncated marker segment");
    }

    let length = read_u16_be(data, marker_index + 1)?;
    if length < 2 {
        bail!("Invalid marker length {}", length);
    }

    let payload_len = usize::from(length - 2);
    let payload_start = marker_index + 3;
    let end = payload_start + payload_len;

    if end > data.len() {
        bail!("Marker segment extends past end of file");
    }

    Ok(end)
}

fn read_u16_be(data: &[u8], index: usize) -> Result<u16> {
    if index + 1 >= data.len() {
        bail!("Unexpected EOF while reading u16");
    }
    Ok(u16::from_be_bytes([data[index], data[index + 1]]))
}

fn parse_sof(payload: &[u8]) -> Result<FrameInfo> {
    if payload.len() < 6 {
        bail!("Invalid SOF payload length");
    }

    let precision = payload[0];
    if precision != 8 {
        bail!(
            "Only 8-bit JPEG precision is supported with --zero-final-ac (found {})",
            precision
        );
    }

    let height = u16::from_be_bytes([payload[1], payload[2]]);
    let width = u16::from_be_bytes([payload[3], payload[4]]);
    let component_count = payload[5] as usize;

    if component_count == 0 {
        bail!("SOF has zero components");
    }

    let expected_len = 6 + component_count * 3;
    if payload.len() != expected_len {
        bail!(
            "Invalid SOF payload length for {} components",
            component_count
        );
    }

    let mut components = Vec::with_capacity(component_count);
    let mut max_h_samp = 1;
    let mut max_v_samp = 1;

    for i in 0..component_count {
        let base = 6 + i * 3;
        let identifier = payload[base];
        let hv = payload[base + 1];
        let h_samp = hv >> 4;
        let v_samp = hv & 0x0F;

        if h_samp == 0 || v_samp == 0 {
            bail!("Invalid sampling factors in SOF");
        }

        max_h_samp = max_h_samp.max(h_samp);
        max_v_samp = max_v_samp.max(v_samp);

        components.push(FrameComponent {
            identifier,
            h_samp,
            v_samp,
        });
    }

    Ok(FrameInfo {
        width,
        height,
        components,
        max_h_samp,
        max_v_samp,
    })
}

fn parse_sos(payload: &[u8], frame: &FrameInfo) -> Result<ScanInfo> {
    if payload.len() < 6 {
        bail!("Invalid SOS payload length");
    }

    let component_count = payload[0] as usize;
    if component_count == 0 {
        bail!("SOS has zero components");
    }

    let expected_min_len = 1 + component_count * 2 + 3;
    if payload.len() != expected_min_len {
        bail!(
            "Invalid SOS payload length for {} components",
            component_count
        );
    }

    let mut components = Vec::with_capacity(component_count);

    for i in 0..component_count {
        let base = 1 + i * 2;
        let id = payload[base];
        let table_selectors = payload[base + 1];
        let dc_table = (table_selectors >> 4) as usize;
        let ac_table = (table_selectors & 0x0F) as usize;

        if dc_table > 3 || ac_table > 3 {
            bail!("Invalid Huffman table index in SOS");
        }

        let frame_component = frame
            .components
            .iter()
            .find(|c| c.identifier == id)
            .ok_or_else(|| anyhow!("SOS references unknown component id {}", id))?;

        components.push(ScanComponent {
            dc_table,
            ac_table,
            h_samp: frame_component.h_samp,
            v_samp: frame_component.v_samp,
        });
    }

    let spectral_start = payload[1 + component_count * 2];
    let spectral_end = payload[1 + component_count * 2 + 1];
    let approx = payload[1 + component_count * 2 + 2];
    let approx_high = approx >> 4;
    let approx_low = approx & 0x0F;

    if spectral_start != 0 || spectral_end != 63 || approx_high != 0 || approx_low != 0 {
        bail!(
            "Only baseline sequential scans are supported with --zero-final-ac (Ss={}, Se={}, Ah={}, Al={})",
            spectral_start,
            spectral_end,
            approx_high,
            approx_low
        );
    }

    Ok(ScanInfo { components })
}

fn parse_dri(payload: &[u8]) -> Result<usize> {
    if payload.len() != 2 {
        bail!("Invalid DRI payload length");
    }

    Ok(u16::from_be_bytes([payload[0], payload[1]]) as usize)
}

fn parse_dht(
    payload: &[u8],
    dc_tables: &mut [Option<HuffmanCoding>; 4],
    ac_tables: &mut [Option<HuffmanCoding>; 4],
) -> Result<()> {
    let mut index = 0;

    while index < payload.len() {
        let table_info = *payload
            .get(index)
            .ok_or_else(|| anyhow!("Invalid DHT payload"))?;
        index += 1;

        let class = table_info >> 4;
        let table_id = (table_info & 0x0F) as usize;

        if class > 1 || table_id > 3 {
            bail!("Invalid DHT table specifier");
        }

        if index + 16 > payload.len() {
            bail!("Invalid DHT code-length section");
        }

        let mut code_lengths = [0u8; 16];
        code_lengths.copy_from_slice(&payload[index..index + 16]);
        index += 16;

        let symbol_count: usize = code_lengths.iter().map(|&v| v as usize).sum();
        if index + symbol_count > payload.len() {
            bail!("Invalid DHT symbol section");
        }

        let symbols = &payload[index..index + symbol_count];
        index += symbol_count;

        let table = build_huffman_table(&code_lengths, symbols)?;

        if class == 0 {
            dc_tables[table_id] = Some(table);
        } else {
            ac_tables[table_id] = Some(table);
        }
    }

    Ok(())
}

fn build_huffman_table(code_lengths: &[u8; 16], symbols: &[u8]) -> Result<HuffmanCoding> {
    let mut decode = HashMap::new();
    let mut encode: [Option<(u16, u8)>; 256] = [None; 256];

    let mut code: u16 = 0;
    let mut symbol_index = 0usize;

    for bit_len in 1..=16 {
        let count = code_lengths[bit_len - 1] as usize;

        for _ in 0..count {
            let symbol = *symbols
                .get(symbol_index)
                .ok_or_else(|| anyhow!("Invalid canonical Huffman table"))?;

            decode.insert((bit_len as u8, code), symbol);
            encode[symbol as usize] = Some((code, bit_len as u8));

            code = code
                .checked_add(1)
                .ok_or_else(|| anyhow!("Huffman code overflow"))?;
            symbol_index += 1;
        }

        code <<= 1;
    }

    if symbol_index != symbols.len() {
        bail!("Unexpected Huffman symbol count");
    }

    Ok(HuffmanCoding { decode, encode })
}

#[derive(Debug)]
enum EntropyReadError {
    Marker { marker: u8, position: usize },
    Eof,
}

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buffer: u8,
    bits_left: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_buffer: 0,
            bits_left: 0,
        }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn byte_align(&mut self) {
        self.bits_left = 0;
    }

    fn read_bits(&mut self, bits: u8) -> std::result::Result<u16, EntropyReadError> {
        let mut out: u16 = 0;
        for _ in 0..bits {
            out <<= 1;
            out |= self.read_bit()? as u16;
        }
        Ok(out)
    }

    fn read_bit(&mut self) -> std::result::Result<u8, EntropyReadError> {
        if self.bits_left == 0 {
            self.bit_buffer = self.read_entropy_byte()?;
            self.bits_left = 8;
        }

        self.bits_left -= 1;
        Ok((self.bit_buffer >> self.bits_left) & 1)
    }

    fn read_entropy_byte(&mut self) -> std::result::Result<u8, EntropyReadError> {
        if self.pos >= self.data.len() {
            return Err(EntropyReadError::Eof);
        }

        let byte = self.data[self.pos];
        self.pos += 1;

        if byte != 0xFF {
            return Ok(byte);
        }

        if self.pos >= self.data.len() {
            return Err(EntropyReadError::Eof);
        }

        let marker = self.data[self.pos];
        self.pos += 1;

        if marker == 0x00 {
            return Ok(0xFF);
        }

        Err(EntropyReadError::Marker {
            marker,
            position: self.pos - 2,
        })
    }

    fn consume_restart_marker(&mut self, expected_index: u8) -> Result<()> {
        self.byte_align();

        if self.pos >= self.data.len() {
            bail!("Unexpected EOF while reading restart marker");
        }

        if self.data[self.pos] != 0xFF {
            bail!(
                "Expected restart marker prefix at byte {}, found 0x{:02X}",
                self.pos,
                self.data[self.pos]
            );
        }

        let mut marker_pos = self.pos + 1;
        while marker_pos < self.data.len() && self.data[marker_pos] == 0xFF {
            marker_pos += 1;
        }

        if marker_pos >= self.data.len() {
            bail!("Unexpected EOF while reading restart marker code");
        }

        let marker = self.data[marker_pos];
        let expected = 0xD0 + (expected_index & 0x07);
        if marker != expected {
            bail!(
                "Expected restart marker 0xFF{:02X}, found 0xFF{:02X}",
                expected,
                marker
            );
        }

        self.pos = marker_pos + 1;
        Ok(())
    }
}

struct BitWriter {
    out: Vec<u8>,
    bit_buffer: u8,
    bits_filled: u8,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            out: Vec::new(),
            bit_buffer: 0,
            bits_filled: 0,
        }
    }

    fn into_bytes(self) -> Vec<u8> {
        self.out
    }

    fn write_bits(&mut self, bits: u16, bit_count: u8) {
        for i in (0..bit_count).rev() {
            let bit = ((bits >> i) & 1) as u8;
            self.bit_buffer = (self.bit_buffer << 1) | bit;
            self.bits_filled += 1;

            if self.bits_filled == 8 {
                self.push_entropy_byte(self.bit_buffer);
                self.bit_buffer = 0;
                self.bits_filled = 0;
            }
        }
    }

    fn byte_align_with_ones(&mut self) {
        if self.bits_filled == 0 {
            return;
        }

        let remaining = 8 - self.bits_filled;
        self.bit_buffer = (self.bit_buffer << remaining) | ((1u8 << remaining) - 1);
        self.push_entropy_byte(self.bit_buffer);
        self.bit_buffer = 0;
        self.bits_filled = 0;
    }

    fn write_restart_marker(&mut self, index: u8) {
        self.byte_align_with_ones();
        self.out.push(0xFF);
        self.out.push(0xD0 + (index & 0x07));
    }

    fn push_entropy_byte(&mut self, byte: u8) {
        self.out.push(byte);
        if byte == 0xFF {
            self.out.push(0x00);
        }
    }
}

fn rewrite_scan_entropy(
    entropy: &[u8],
    frame: &FrameInfo,
    scan: &ScanInfo,
    dc_tables: &[Option<HuffmanCoding>; 4],
    ac_tables: &[Option<HuffmanCoding>; 4],
    restart_interval: usize,
) -> Result<(Vec<u8>, usize)> {
    let mcu_width = 8usize * frame.max_h_samp as usize;
    let mcu_height = 8usize * frame.max_v_samp as usize;
    let mcus_x = (frame.width as usize).div_ceil(mcu_width);
    let mcus_y = (frame.height as usize).div_ceil(mcu_height);
    let total_mcus = mcus_x * mcus_y;

    let mut reader = BitReader::new(entropy);
    let mut writer = BitWriter::new();

    let mut dc_predictors_decode = vec![0i16; scan.components.len()];
    let mut dc_predictors_encode = vec![0i16; scan.components.len()];

    let mut restart_index: u8 = 0;

    for mcu_index in 0..total_mcus {
        for (scan_component_index, component) in scan.components.iter().enumerate() {
            let dc_table = dc_tables
                .get(component.dc_table)
                .and_then(Option::as_ref)
                .ok_or_else(|| anyhow!("Missing DC Huffman table {}", component.dc_table))?;

            let ac_table = ac_tables
                .get(component.ac_table)
                .and_then(Option::as_ref)
                .ok_or_else(|| anyhow!("Missing AC Huffman table {}", component.ac_table))?;

            let blocks_in_component = usize::from(component.h_samp) * usize::from(component.v_samp);

            for _ in 0..blocks_in_component {
                let mut coefficients = decode_block(
                    &mut reader,
                    &mut dc_predictors_decode[scan_component_index],
                    dc_table,
                    ac_table,
                )?;

                coefficients[63] = 0;

                encode_block(
                    &mut writer,
                    &coefficients,
                    &mut dc_predictors_encode[scan_component_index],
                    dc_table,
                    ac_table,
                )?;
            }
        }

        if restart_interval > 0
            && (mcu_index + 1) % restart_interval == 0
            && (mcu_index + 1) < total_mcus
        {
            reader.consume_restart_marker(restart_index)?;
            writer.write_restart_marker(restart_index);

            dc_predictors_decode.fill(0);
            dc_predictors_encode.fill(0);
            restart_index = (restart_index + 1) & 0x07;
        }
    }

    writer.byte_align_with_ones();

    let marker_offset = find_next_entropy_marker(entropy, reader.position())?;
    Ok((writer.into_bytes(), marker_offset))
}

fn decode_block(
    reader: &mut BitReader<'_>,
    dc_predictor: &mut i16,
    dc_table: &HuffmanCoding,
    ac_table: &HuffmanCoding,
) -> Result<[i16; 64]> {
    let mut coefficients = [0i16; 64];

    let dc_size = decode_huffman_symbol(reader, dc_table)?;
    let dc_delta = if dc_size == 0 {
        0
    } else {
        let bits = reader.read_bits(dc_size).map_err(map_entropy_read_error)?;
        receive_extend(bits, dc_size)
    };

    *dc_predictor += dc_delta;
    coefficients[0] = *dc_predictor;

    let mut index = 1usize;
    while index < 64 {
        let symbol = decode_huffman_symbol(reader, ac_table)?;

        if symbol == 0x00 {
            break;
        }

        if symbol == 0xF0 {
            index += 16;
            continue;
        }

        let run_length = (symbol >> 4) as usize;
        let value_size = symbol & 0x0F;

        index += run_length;
        if index >= 64 {
            bail!("Invalid AC run length in JPEG scan");
        }

        let bits = reader
            .read_bits(value_size)
            .map_err(map_entropy_read_error)?;
        coefficients[index] = receive_extend(bits, value_size);
        index += 1;
    }

    Ok(coefficients)
}

fn encode_block(
    writer: &mut BitWriter,
    coefficients: &[i16; 64],
    dc_predictor: &mut i16,
    dc_table: &HuffmanCoding,
    ac_table: &HuffmanCoding,
) -> Result<()> {
    let dc_delta = coefficients[0] - *dc_predictor;
    *dc_predictor = coefficients[0];

    let dc_size = magnitude_category(dc_delta);
    write_huffman_symbol(writer, dc_table, dc_size)?;

    if dc_size > 0 {
        let bits = coefficient_bits(dc_delta, dc_size)?;
        writer.write_bits(bits, dc_size);
    }

    let mut zero_run = 0usize;
    for coefficient in coefficients.iter().skip(1) {
        if *coefficient == 0 {
            zero_run += 1;
            continue;
        }

        while zero_run >= 16 {
            write_huffman_symbol(writer, ac_table, 0xF0)?;
            zero_run -= 16;
        }

        let ac_size = magnitude_category(*coefficient);
        let symbol = ((zero_run as u8) << 4) | ac_size;
        write_huffman_symbol(writer, ac_table, symbol)?;

        let bits = coefficient_bits(*coefficient, ac_size)?;
        writer.write_bits(bits, ac_size);

        zero_run = 0;
    }

    if zero_run > 0 {
        write_huffman_symbol(writer, ac_table, 0x00)?;
    }

    Ok(())
}

fn decode_huffman_symbol(reader: &mut BitReader<'_>, table: &HuffmanCoding) -> Result<u8> {
    let mut code = 0u16;

    for bit_len in 1..=16u8 {
        let bit = reader.read_bits(1).map_err(map_entropy_read_error)?;
        code = (code << 1) | bit;

        if let Some(symbol) = table.decode.get(&(bit_len, code)) {
            return Ok(*symbol);
        }
    }

    bail!("Failed to decode Huffman symbol")
}

fn write_huffman_symbol(writer: &mut BitWriter, table: &HuffmanCoding, symbol: u8) -> Result<()> {
    let (code, bit_len) = table.encode[symbol as usize]
        .ok_or_else(|| anyhow!("Missing Huffman encoding for symbol 0x{:02X}", symbol))?;
    writer.write_bits(code, bit_len);
    Ok(())
}

fn magnitude_category(value: i16) -> u8 {
    if value == 0 {
        return 0;
    }

    let mut magnitude = value.unsigned_abs();
    let mut size = 0u8;
    while magnitude > 0 {
        magnitude >>= 1;
        size += 1;
    }
    size
}

fn coefficient_bits(value: i16, size: u8) -> Result<u16> {
    if size == 0 {
        return Ok(0);
    }

    let limit = (1i32 << size) - 1;

    if value >= 0 {
        Ok(value as u16)
    } else {
        let encoded = value as i32 + limit;
        if encoded < 0 {
            bail!("Coefficient out of range for size {}", size);
        }
        Ok(encoded as u16)
    }
}

fn receive_extend(bits: u16, size: u8) -> i16 {
    if size == 0 {
        return 0;
    }

    let vt = 1u16 << (size - 1);
    if bits < vt {
        bits as i16 - ((1i16 << size) - 1)
    } else {
        bits as i16
    }
}

fn map_entropy_read_error(err: EntropyReadError) -> anyhow::Error {
    match err {
        EntropyReadError::Marker { marker, position } => anyhow!(
            "Unexpected marker 0xFF{:02X} inside entropy-coded scan at byte {}",
            marker,
            position
        ),
        EntropyReadError::Eof => anyhow!("Unexpected EOF in entropy-coded scan"),
    }
}

fn find_next_entropy_marker(data: &[u8], start: usize) -> Result<usize> {
    if start >= data.len() {
        bail!("Unable to find marker after entropy-coded scan");
    }

    let mut index = start;
    while index + 1 < data.len() {
        if data[index] != 0xFF {
            index += 1;
            continue;
        }

        let mut marker_index = index + 1;
        while marker_index < data.len() && data[marker_index] == 0xFF {
            marker_index += 1;
        }

        if marker_index >= data.len() {
            break;
        }

        let marker = data[marker_index];
        match marker {
            0x00 => {
                index = marker_index + 1;
            }
            0xD0..=0xD7 => {
                index = marker_index + 1;
            }
            _ => return Ok(index),
        }
    }

    bail!("Unable to locate marker after entropy-coded scan")
}
