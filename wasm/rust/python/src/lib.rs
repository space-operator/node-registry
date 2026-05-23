use rustpython_vm::{builtins, stdlib::get_module_inits, Interpreter};
use serde::Deserialize;
use space_lib::space;

#[derive(Deserialize)]
struct Input {
    script: String,
}

#[space]
fn main(mut input: Input) -> String {
    Interpreter::with_init(Default::default(), |vm| {
        vm.add_native_modules(get_module_inits());
    })
    .enter(|vm| {
        let scope = vm.new_scope_with_builtins();

        vm.run_block_expr(scope, &input.script)
            .unwrap()
            .downcast::<builtins::PyStr>()
            .map(|s| s.to_string())
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let input = Input {
            script: String::from("1+1"),
        };
        let output: String = main(input);
        // dbg!(output);
        // panic!();
    }
}
