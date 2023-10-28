# Optimisers

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/KGrewal1/optimisers/graph/badge.svg?token=6AFTLS6DFO)](https://codecov.io/gh/KGrewal1/optimisers)
![Tests](https://github.com/KGrewal1/optimisers/actions/workflows/rust-ci.yml/badge.svg)

A crate for optimisers for use with [candle](https://github.com/huggingface/candle), the minimalist ML framework

* Momentum enhanced SGD

* AdaGrad

* AdaDelta

* AdaMax

* Adam

* AdamW (included with Adam as `decoupled_weight_decay`)

* NAdam

* RAdam

* RMSprop

These are all checked against their pytorch implementation (see pytorch_test.ipynb) and should implement the same functionality (though without some input checking).

## Examples

There is an mnist toy program along with a simple example of adagrad. Whilst the parameters of each method aren't tuned (all default with user input learning rate), the following converges quite nicely:

```cli
cargo r -r --example mnist mlp --optim r-adam --epochs 2000 --learning-rate 0.025
```

For even faster training try:

```cli
cargo r -r --features cuda --example mnist mlp --optim r-adam --epochs 2000 --learning-rate 0.025
```

to use the cuda backend.

## Usage

```cli
cargo add --git https://github.com/KGrewal1/optimisers.git optimisers
```

## To do

Currently unimplemented from pytorch:

* SparseAdam (unsure how to treat sparse tensors in candle)

* ASGD (no pseudocode)

* LBFGS (need to reformulate in terms of tensors / no pseudocode)

* Rprop (need to reformulate in terms of tensors)

## Notes

For development, to track state of pytorch methods, use:

```python
print(optimiser.state)
```
