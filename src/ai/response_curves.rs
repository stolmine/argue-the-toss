// Response Curves for Utility AI
// Maps input values [0.0, 1.0] to output scores [0.0, 1.0]

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseCurve {
    Linear,
    Polynomial { exponent: f32 },
    Logistic { midpoint: f32, steepness: f32 },
    Inverse,
    InverseSquared,
    Boolean { threshold: f32 },
    Step { thresholds: Vec<(f32, f32)> },
}

impl ResponseCurve {
    pub fn evaluate(&self, x: f32) -> f32 {
        let x_clamped = x.clamp(0.0, 1.0);

        let result = match self {
            ResponseCurve::Linear => x_clamped,

            ResponseCurve::Polynomial { exponent } => {
                x_clamped.powf(*exponent)
            }

            ResponseCurve::Logistic { midpoint, steepness } => {
                1.0 / (1.0 + (-steepness * (x_clamped - midpoint)).exp())
            }

            ResponseCurve::Inverse => {
                1.0 - x_clamped
            }

            ResponseCurve::InverseSquared => {
                (1.0 - x_clamped).powi(2)
            }

            ResponseCurve::Boolean { threshold } => {
                if x_clamped >= *threshold { 1.0 } else { 0.0 }
            }

            ResponseCurve::Step { thresholds } => {
                if thresholds.is_empty() {
                    return 0.0;
                }

                if thresholds.len() == 1 {
                    return thresholds[0].1;
                }

                for i in 0..thresholds.len() - 1 {
                    let (x1, y1) = thresholds[i];
                    let (x2, y2) = thresholds[i + 1];

                    if x_clamped >= x1 && x_clamped < x2 {
                        let t = (x_clamped - x1) / (x2 - x1);
                        return y1 + t * (y2 - y1);
                    }
                }

                thresholds.last().unwrap().1
            }
        };

        result.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    fn assert_near(a: f32, b: f32) {
        assert!((a - b).abs() < EPSILON, "Expected {} to be near {}", a, b);
    }

    #[test]
    fn test_linear_curve() {
        let curve = ResponseCurve::Linear;

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.5), 0.5);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_linear_clamping() {
        let curve = ResponseCurve::Linear;

        assert_near(curve.evaluate(-0.5), 0.0);
        assert_near(curve.evaluate(1.5), 1.0);
    }

    #[test]
    fn test_polynomial_quadratic() {
        let curve = ResponseCurve::Polynomial { exponent: 2.0 };

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.5), 0.25);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_polynomial_cubic() {
        let curve = ResponseCurve::Polynomial { exponent: 3.0 };

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.5), 0.125);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_logistic_midpoint() {
        let curve = ResponseCurve::Logistic {
            midpoint: 0.5,
            steepness: 10.0,
        };

        assert_near(curve.evaluate(0.5), 0.5);
        assert!(curve.evaluate(0.0) < 0.1);
        assert!(curve.evaluate(1.0) > 0.9);
    }

    #[test]
    fn test_logistic_steep() {
        let curve = ResponseCurve::Logistic {
            midpoint: 0.5,
            steepness: 20.0,
        };

        assert!(curve.evaluate(0.0) < 0.01);
        assert_near(curve.evaluate(0.5), 0.5);
        assert!(curve.evaluate(1.0) > 0.99);
    }

    #[test]
    fn test_inverse_curve() {
        let curve = ResponseCurve::Inverse;

        assert_near(curve.evaluate(0.0), 1.0);
        assert_near(curve.evaluate(0.5), 0.5);
        assert_near(curve.evaluate(1.0), 0.0);
    }

    #[test]
    fn test_inverse_squared_curve() {
        let curve = ResponseCurve::InverseSquared;

        assert_near(curve.evaluate(0.0), 1.0);
        assert_near(curve.evaluate(0.5), 0.25);
        assert_near(curve.evaluate(1.0), 0.0);
    }

    #[test]
    fn test_boolean_threshold() {
        let curve = ResponseCurve::Boolean { threshold: 0.7 };

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.5), 0.0);
        assert_near(curve.evaluate(0.69), 0.0);
        assert_near(curve.evaluate(0.7), 1.0);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_step_empty() {
        let curve = ResponseCurve::Step { thresholds: vec![] };
        assert_near(curve.evaluate(0.5), 0.0);
    }

    #[test]
    fn test_step_single() {
        let curve = ResponseCurve::Step {
            thresholds: vec![(0.0, 0.8)],
        };

        assert_near(curve.evaluate(0.0), 0.8);
        assert_near(curve.evaluate(0.5), 0.8);
        assert_near(curve.evaluate(1.0), 0.8);
    }

    #[test]
    fn test_step_interpolation() {
        let curve = ResponseCurve::Step {
            thresholds: vec![
                (0.0, 0.0),
                (0.5, 0.5),
                (1.0, 1.0),
            ],
        };

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.25), 0.25);
        assert_near(curve.evaluate(0.5), 0.5);
        assert_near(curve.evaluate(0.75), 0.75);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_step_non_linear() {
        let curve = ResponseCurve::Step {
            thresholds: vec![
                (0.0, 0.0),
                (0.3, 0.1),
                (0.7, 0.9),
                (1.0, 1.0),
            ],
        };

        assert_near(curve.evaluate(0.0), 0.0);
        assert_near(curve.evaluate(0.15), 0.05);
        assert_near(curve.evaluate(0.5), 0.5);
        assert_near(curve.evaluate(0.85), 0.95);
        assert_near(curve.evaluate(1.0), 1.0);
    }

    #[test]
    fn test_polynomial_monotonic_increasing() {
        let curve = ResponseCurve::Polynomial { exponent: 2.0 };

        let v1 = curve.evaluate(0.3);
        let v2 = curve.evaluate(0.5);
        let v3 = curve.evaluate(0.7);

        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn test_inverse_monotonic_decreasing() {
        let curve = ResponseCurve::Inverse;

        let v1 = curve.evaluate(0.3);
        let v2 = curve.evaluate(0.5);
        let v3 = curve.evaluate(0.7);

        assert!(v1 > v2);
        assert!(v2 > v3);
    }

    #[test]
    fn test_logistic_boundaries() {
        let curve = ResponseCurve::Logistic {
            midpoint: 0.5,
            steepness: 10.0,
        };

        let v_min = curve.evaluate(0.0);
        let v_max = curve.evaluate(1.0);

        assert!(v_min >= 0.0 && v_min <= 1.0);
        assert!(v_max >= 0.0 && v_max <= 1.0);
    }
}
