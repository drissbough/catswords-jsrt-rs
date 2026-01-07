extern crate catswords_jsrt as js;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create runtime
    let runtime = js::Runtime::new()?;

    // 2. Create context
    let context = js::Context::new(&runtime)?;

    // 3. Make context current (RAII)
    let guard = context.make_current()?;

    // 4. Evaluate script
    let result = js::script::eval(&guard, "5 + 5")?;

    // 5. Convert result
    let value = result.to_integer(&guard)?;

    assert_eq!(value, 10);
    println!("5 + 5 = {}", value);

    Ok(())
}
