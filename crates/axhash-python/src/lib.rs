use axhash_core::{AxHasher, RuntimeBackend, axhash, axhash_seeded, runtime_backend, runtime_has_aes};
use pyo3::prelude::*;

fn backend_name(backend: RuntimeBackend) -> &'static str {
    match backend {
        RuntimeBackend::Scalar => "scalar",
        RuntimeBackend::Aarch64AesNeon => "aarch64-aes-neon",
        RuntimeBackend::X86_64AesAvx2 => "x86_64-aes-avx2",
    }
}

#[pyfunction(name = "axhash")]
fn axhash_py(data: &[u8]) -> u64 {
    axhash(data)
}

#[pyfunction(name = "axhash_seeded")]
fn axhash_seeded_py(data: &[u8], seed: u64) -> u64 {
    axhash_seeded(data, seed)
}

#[pyfunction(name = "runtime_backend")]
fn runtime_backend_py() -> &'static str {
    backend_name(runtime_backend())
}

#[pyfunction(name = "runtime_has_aes")]
fn runtime_has_aes_py() -> bool {
    runtime_has_aes()
}

#[pyclass]
struct Hasher {
    inner: AxHasher,
}

#[pymethods]
impl Hasher {
    #[new]
    #[pyo3(signature = (seed=0))]
    fn new(seed: u64) -> Self {
        Self {
            inner: AxHasher::new_with_seed(seed),
        }
    }

    fn reset(&mut self, seed: u64) {
        self.inner = AxHasher::new_with_seed(seed);
    }

    fn update(&mut self, data: &[u8]) {
        use core::hash::Hasher as _;
        self.inner.write(data);
    }

    fn digest(&self) -> u64 {
        use core::hash::Hasher as _;
        self.inner.finish()
    }
}

#[pymodule]
fn _axhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Hasher>()?;
    m.add_function(wrap_pyfunction!(axhash_py, m)?)?;
    m.add_function(wrap_pyfunction!(axhash_seeded_py, m)?)?;
    m.add_function(wrap_pyfunction!(runtime_backend_py, m)?)?;
    m.add_function(wrap_pyfunction!(runtime_has_aes_py, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
