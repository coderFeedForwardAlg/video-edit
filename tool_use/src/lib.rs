use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn subtract_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a - b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn tool_use(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(subtract_as_string, m)?)?;
    Ok(())
}
