

pub trait VecMath {
    fn sum(&self) -> f32;
    fn mean(&self) -> f32;
    fn variance(&self) -> f32;
    fn xcorr(&self, other:&Self) -> f32;
    fn stddev(&self) -> f32;
    fn z_score(&self, check_value:f32) -> f32;
}

impl VecMath for Vec<f32> {

    fn sum(&self) -> f32 {
        let mut s = 0.0;
        for v in self.iter() {
            s += v;
        }
        s
    }

    fn mean(&self) -> f32 {
        self.sum() / self.len() as f32
    }
    
    fn variance(&self) -> f32 {
        let m = self.mean();

        let mut sqdiff = 0.0;
        for v in self.iter() {
            sqdiff += (v - m) * (v - m);
        }
        sqdiff / self.len() as f32
    }



    fn xcorr(&self, other:&Self) -> f32 {
        if self.len() != other.len() {
            panic!("Arrays need to be the same length (for now)");
        }
        let m_x = self.mean();
        let m_y = other.mean();
        let v_x = self.variance();
        let v_y = other.variance();

        let mut s = 0.0;
        for n in 0..self.len() {
            s += (self[n] - m_x) * (other[n] - m_y)
        }
        let c = 1.0 / self.len() as f32 * s / (v_x * v_y).sqrt();

        c
    }

    fn stddev(&self) -> f32 {
        self.variance().sqrt()
    }

    fn z_score(&self, check_value:f32) -> f32 {
        (check_value - self.mean()) / self.stddev()
    }

}

