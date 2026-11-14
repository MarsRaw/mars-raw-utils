use crate::subs::runnable::RunnableSubcommand;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;

use std::fs::File;
use std::io::Write;

use stump;

use std::io::{BufRead, BufReader, BufWriter};

pb_create_spinner!();

/// Offset a single 1-based index. Empty fields and negative (relative)
/// indices are left alone.
fn adjust(s: &str, off: i64) -> Result<String> {
    if s.is_empty() {
        return Ok(String::new());
    }
    let i: i64 = s.parse().map_err(|e| anyhow!("bad index {s:?}: {e}"))?;
    Ok(if i > 0 {
        (i + off).to_string()
    } else {
        i.to_string()
    })
}

/// Handle a face/line/point token of the form v, v/vt, v//vn or v/vt/vn.
fn adjust_token(tok: &str, v_off: i64, vt_off: i64, vn_off: i64) -> Result<String> {
    let mut parts: Vec<String> = Vec::with_capacity(3);
    for (n, field) in tok.split('/').enumerate() {
        let off = match n {
            0 => v_off,
            1 => vt_off,
            2 => vn_off,
            _ => return Err(anyhow!("too many fields in token {tok:?}")),
        };
        parts.push(adjust(field, off)?);
    }
    Ok(parts.join("/"))
}

#[derive(Parser)]
#[command(author, version, about = "Merge OBJ files", long_about = None)]
pub struct MergeMesh {
    #[arg(long, short, help = "Input meshes", num_args = 1..)]
    input_files: Vec<std::path::PathBuf>,

    #[arg(long, short, help = "Output mesh (.obj)")]
    output: std::path::PathBuf,
}

impl RunnableSubcommand for MergeMesh {
    async fn run(&self) -> Result<()> {
        pb_set_print!();
        stump::print_experimental();

        if let Some(parent) = self.output.parent()
            && parent.is_absolute()
            && (!parent.exists() || parent.metadata().unwrap().permissions().readonly())
        {
            return Err(anyhow!(
                "Error: output directory does not exist or is not writable: {:?}",
                parent
            ));
        }

        let mut out = BufWriter::new(File::create(&self.output)?);
        let (mut v_off, mut vt_off, mut vn_off) = (0i64, 0i64, 0i64);

        for p in &self.input_files {
            if !p.exists() {
                return Err(anyhow!("Error: input file not found: {:?}", p));
            }

            let (mut nv, mut nvt, mut nvn) = (0i64, 0i64, 0i64);
            let reader = BufReader::new(File::open(p)?);

            for line in reader.lines() {
                let line = line?;
                let mut words = line.split_whitespace();
                match words.next() {
                    Some("v") => {
                        nv += 1;
                        writeln!(out, "{line}")?;
                    }
                    Some("vt") => {
                        nvt += 1;
                        writeln!(out, "{line}")?;
                    }
                    Some("vn") => {
                        nvn += 1;
                        writeln!(out, "{line}")?;
                    }
                    // Faces, lines and points all reference vertex indices.
                    Some(kw @ ("f" | "l" | "p")) => {
                        let toks: Result<Vec<String>> = words
                            .map(|t| adjust_token(t, v_off, vt_off, vn_off))
                            .collect();
                        writeln!(out, "{kw} {}", toks?.join(" "))?;
                    }
                    // o, g, s, usemtl, mtllib, comments, blank lines, etc.
                    _ => writeln!(out, "{line}")?,
                }
            }
            v_off += nv;
            vt_off += nvt;
            vn_off += nvn;
        }

        pb_done!();
        Ok(())
    }
}
