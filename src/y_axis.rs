use std::cmp::{self, max};

use itertools::Itertools;
use ordered_float::OrderedFloat;

use crate::Float;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Numeric {
    precision: usize,
    scale: usize,
}

impl Default for Numeric {
    fn default() -> Self {
        Self::new(9, 3)
    }
}

impl Numeric {
    pub fn new(precision: usize, scale: usize) -> Self {
        Self { precision, scale }
    }

    pub fn format(&self, value: Float) -> String {
        let precision = self.precision;
        let scale = self.scale;
        format!("{0:>precision$.scale$}", value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Grid {
    Accurate,
    Readable,
}

impl Default for Grid {
    fn default() -> Self {
        Grid::Accurate
    }
}

pub(crate) struct YAxis {
    numeric: Numeric,
    grid: Grid,
    height: u16,
    min: Float,
    max: Float,
    unit: Float,
}

impl YAxis {
    pub fn estimated_width(numeric: Numeric, min: Float, max: Float) -> u16 {
        cmp::max(numeric.format(max).len(), numeric.format(min).len()) as u16 + 4
    }

    pub fn new(numeric: Numeric, grid: Grid, height: u16, min: Float, max: Float) -> Self {
        assert!(min <= max);
        let unit = (max - min) / OrderedFloat::from(height as f64);

        Self {
            numeric,
            grid,
            height,
            min,
            max,
            unit,
        }
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn calc_y(&self, value: Float) -> Float {
        (value - self.min) / self.unit
    }

    pub fn render(&self) -> Vec<String> {
        let mut result = Vec::new();
        let max_chars = max(
            self.numeric.format(self.max).len(),
            self.numeric.format(self.min).len(),
        );

        match self.grid {
            Grid::Accurate => {
                for i in 0..self.height {
                    let rendered = if i % 4 == 0 {
                        let value = self.max - self.unit * OrderedFloat::from(i);
                        format!(" {} ├ ", self.numeric.format(value))
                    } else {
                        format!(" {} │ ", " ".repeat(max_chars))
                    };

                    result.push(rendered);
                }
            }
            Grid::Readable => {
                let readable_unit = 10f64.powi((*self.unit).log10() as i32);
                // multiple of 1, 5, or 10 * readable_unit
                let min_space = 4;
                let co = (min_space as f64 * *self.unit / readable_unit) as i32;

                let actual_unit = [1, 5, 10, 20, 50, 100]
                    .into_iter()
                    .filter(|c| *c >= co)
                    .next()
                    .unwrap() as f64
                    * readable_unit;

                for (high, low) in (0..=self.height)
                    .into_iter()
                    .map(|i| self.max - self.unit * OrderedFloat::from(i))
                    .tuple_windows()
                {
                    let mid = (*high + *low) / 2.;

                    let mark = (((*low / actual_unit) as i32)..=(*high / actual_unit) as i32)
                        .into_iter()
                        .map(|c| c as f64 * actual_unit)
                        .filter(|mark| *low <= *mark && *mark <= *high)
                        .map(|mark| (OrderedFloat::from((mark - mid).abs()), mark))
                        .sorted_by(|a, b| a.0.cmp(&b.0))
                        .next();

                    let rendered = if let Some((_, mark)) = mark {
                        format!(" {} ├ ", self.numeric.format(mark.into()))
                    } else {
                        format!(" {} │ ", " ".repeat(max_chars))
                    };

                    result.push(rendered);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::Float;

    #[test]
    fn test_format() {
        let numeric = Numeric::new(10, 2);
        assert_eq!(numeric.format(Float::from(3.123456)), "      3.12");
        assert_eq!(numeric.format(Float::from(99991)), "  99991.00");
    }

    #[test]
    fn test_calc() {
        let y_axis = YAxis::new(
            Numeric::default(),
            Grid::default(),
            40,
            100.into(),
            200.into(),
        );
        assert_eq!(y_axis.calc_y(130.into()), OrderedFloat::from(12));
    }
}
