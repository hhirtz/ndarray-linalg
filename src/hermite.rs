//! Define trait for Hermite matrices

use ndarray::{Ix2, Array};
use lapack::c::Layout;

use matrix::{Matrix, MFloat};
use square::SquareMatrix;
use error::LinalgError;
use eigh::ImplEigh;
use cholesky::ImplCholesky;

pub trait HMFloat: ImplEigh + ImplCholesky + MFloat {}
impl HMFloat for f32 {}
impl HMFloat for f64 {}

/// Methods for Hermite matrix
pub trait HermiteMatrix: SquareMatrix + Matrix {
    /// eigenvalue decomposition
    fn eigh(self) -> Result<(Self::Vector, Self), LinalgError>;
    /// symmetric square root of Hermite matrix
    fn ssqrt(self) -> Result<Self, LinalgError>;
    /// Cholesky factorization
    fn cholesky(self) -> Result<Self, LinalgError>;
    /// calc determinant using Cholesky factorization
    fn deth(self) -> Result<Self::Scalar, LinalgError>;
}

impl<A: HMFloat> HermiteMatrix for Array<A, Ix2> {
    fn eigh(self) -> Result<(Self::Vector, Self), LinalgError> {
        self.check_square()?;
        let layout = self.layout()?;
        let (rows, cols) = self.size();
        let (w, a) = ImplEigh::eigh(layout, rows, self.into_raw_vec())?;
        let ea = Array::from_vec(w);
        let va = match layout {
            Layout::ColumnMajor => Array::from_vec(a).into_shape((rows, cols)).unwrap().reversed_axes(),
            Layout::RowMajor => Array::from_vec(a).into_shape((rows, cols)).unwrap(),
        };
        Ok((ea, va))
    }
    fn ssqrt(self) -> Result<Self, LinalgError> {
        let (n, _) = self.size();
        let (e, v) = self.eigh()?;
        let mut res = Array::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                res[(i, j)] = e[i].sqrt() * v[(j, i)];
            }
        }
        Ok(v.dot(&res))
    }
    fn cholesky(self) -> Result<Self, LinalgError> {
        self.check_square()?;
        let (n, _) = self.size();
        let layout = self.layout()?;
        let a = ImplCholesky::cholesky(layout, n, self.into_raw_vec())?;
        let mut c = match layout {
            Layout::RowMajor => Array::from_vec(a).into_shape((n, n)).unwrap(),
            Layout::ColumnMajor => Array::from_vec(a).into_shape((n, n)).unwrap().reversed_axes(),
        };
        for ((i, j), val) in c.indexed_iter_mut() {
            if i > j {
                *val = A::zero();
            }
        }
        Ok(c)
    }
    fn deth(self) -> Result<Self::Scalar, LinalgError> {
        let (n, _) = self.size();
        let c = self.cholesky()?;
        let rt = (0..n).map(|i| c[(i, i)]).fold(A::one(), |det, c| det * c);
        Ok(rt * rt)
    }
}
