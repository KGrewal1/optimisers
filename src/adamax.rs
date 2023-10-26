use candle_core::{Result, Tensor, TensorId, Var};
use candle_nn::optim::Optimizer;
use std::collections::HashMap;

/// Adamax optimizer
///
/// Described in <https://arxiv.org/abs/1412.6980>
///
/// For pseudocde see <https://pytorch.org/docs/stable/generated/torch.optim.Adamax.html>

#[derive(Debug)]
pub struct Adamax {
    vars: Vec<VarAdaMax>,
    params: ParamsAdaMax,
    moment_norm: HashMap<TensorId, (Tensor, Tensor)>,
    t: f64,
}

#[derive(Debug)]
struct VarAdaMax {
    theta: Var,
    m: Var,
    u: Var,
}

#[derive(Debug)]
pub struct ParamsAdaMax {
    pub lr: f64,
    pub beta_1: f64,
    pub beta_2: f64,
    pub weight_decay: f64,
    pub eps: f64,
}

impl Default for ParamsAdaMax {
    fn default() -> Self {
        Self {
            lr: 1.0,
            beta_1: 0.9,
            beta_2: 0.999,
            weight_decay: 0.0,
            eps: 1e-8,
        }
    }
}

impl Optimizer for Adamax {
    type Config = ParamsAdaMax;

    fn new(vars: Vec<Var>, params: ParamsAdaMax) -> Result<Self> {
        let vars = vars
            .into_iter()
            .filter(|var| var.dtype().is_float())
            .map(|var| {
                let dtype = var.dtype();
                let shape = var.shape();
                let device = var.device();
                let m = Var::zeros(shape, dtype, device)?;
                let u = Var::zeros(shape, dtype, device)?;
                Ok(VarAdaMax { theta: var, m, u })
            })
            .collect::<Result<Vec<VarAdaMax>>>()?;
        // // Err(SGDError::NoMomentum)?;
        // let mut params = params;
        // params.t = 0;
        Ok(Self {
            vars,
            params,
            moment_norm: HashMap::new(),
            t: 1.,
        })
    }

    fn learning_rate(&self) -> f64 {
        self.params.lr
    }

    fn step(&mut self, grads: &candle_core::backprop::GradStore) -> Result<()> {
        for var in &self.vars {
            let theta = &var.theta;
            let m = &var.m;
            let u = &var.u;
            if let Some(grad) = grads.get(theta) {
                if self.params.weight_decay == 0. {
                    let m_next =
                        ((self.params.beta_1 * m.as_tensor())? + (1. - self.params.beta_1) * grad)?;
                    let u_next = (self.params.beta_2 * u.as_tensor())?
                        .maximum(&(grad.abs()? + self.params.eps)?)?;
                    let delta = (&m_next * self.params.lr)?
                        .div(&(&u_next * (1. - self.params.beta_1.powf(self.t)))?)?;
                    theta.set(&theta.sub(&(delta))?)?;
                    m.set(&m_next)?;
                    u.set(&u_next)?;
                    self.moment_norm.insert(theta.id(), (m_next, u_next));
                } else {
                    let grad = &(grad + (self.params.weight_decay * theta.as_tensor())?)?;
                    let m_next =
                        ((self.params.beta_1 * m.as_tensor())? + (1. - self.params.beta_1) * grad)?;
                    let u_next = (self.params.beta_2 * u.as_tensor())?
                        .maximum(&(grad.abs()? + self.params.eps)?)?;
                    let delta = (&m_next * self.params.lr)?
                        .div(&(&u_next * (1. - self.params.beta_1.powf(self.t)))?)?;
                    theta.set(&theta.sub(&(delta))?)?;
                    m.set(&m_next)?;
                    u.set(&u_next)?;
                    self.moment_norm.insert(theta.id(), (m_next, u_next));
                }
            }
        }
        self.t += 1.;
        Ok(())
    }

    fn set_learning_rate(&mut self, lr: f64) {
        self.params.lr = lr;
    }
}

impl Adamax {
    #[must_use]
    pub fn into_inner(self) -> Vec<Var> {
        self.vars.into_iter().map(|v| v.theta).collect()
    }

    // pub fn push(&mut self, var: &Var) {
    //     self.vars.push(var.clone());
    // }
}

#[cfg(test)]
mod tests {
    // use candle_core::test_utils::{to_vec0_round, to_vec2_round};

    use anyhow::Result;
    use assert_approx_eq::assert_approx_eq;
    use candle_core::{Device, Tensor, Var};
    use candle_nn::{Linear, Module, Optimizer};

    use super::*;
    #[test]
    fn insertiontest() -> Result<()> {
        let w_gen = Tensor::new(&[[3f32, 1.]], &Device::Cpu)?;
        let b_gen = Tensor::new(-2f32, &Device::Cpu)?;
        let gen = Linear::new(w_gen, Some(b_gen));
        let sample_xs = Tensor::new(&[[2f32, 1.], [7., 4.], [-4., 12.], [5., 8.]], &Device::Cpu)?;
        let _sample_ys = gen.forward(&sample_xs)?;

        let params = ParamsAdaMax {
            lr: 0.004,
            ..Default::default()
        };
        // Now use backprop to run a linear regression between samples and get the coefficients back.
        let w = Var::new(&[[0f32, 0.]], &Device::Cpu)?;
        let b = Var::new(0f32, &Device::Cpu)?;
        let mut n_sgd = Adamax::new(vec![w.clone(), b.clone()], params)?;
        assert_approx_eq!(0.004, n_sgd.learning_rate());
        n_sgd.set_learning_rate(0.002);
        assert_approx_eq!(0.002, n_sgd.learning_rate());
        Ok(())
    }
}