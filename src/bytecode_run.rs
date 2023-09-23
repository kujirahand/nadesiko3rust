use crate::bytecode_gen::*;
use crate::node::*;

/// バイトコードを実行する
pub fn run(items: &mut BytecodeItems) -> Result<NodeValue, String> {
    items.index = 0;
    while items.index < items.codes.len() {
        let code = &items.codes[items.index];
        match code {
            Bytecode::ConstInt(v) => items.stack.push(NodeValue::I(*v)),
            Bytecode::ConstFloat(v) => items.stack.push(NodeValue::F(*v)),
            Bytecode::ConstBool(v) => items.stack.push(NodeValue::B(*v)),
            Bytecode::ConstStr(id) => {
                let s = items.get_string(*id);
                items.stack.push(NodeValue::S(s));
            },
            Bytecode::LetVarGlobal(id) => {
                let name = items.get_string(*id);
                match items.stack.pop() {
                    Some(v) => { items.global_vars.insert(name, v); },
                    None => { items.errors.push(format!("[system] stack not enough")); }
                }
            },
            Bytecode::GetVarGlobal(id) => {
                let name = items.get_string(*id);
                let v = match items.global_vars.get(&name) {
                    Some(v) => v.clone(),
                    None => NodeValue::Empty,
                };
                items.stack.push(v);
            }
            _ => { println!("[ERROR] not implment {:?}", code); },
        }
    }
    Ok(NodeValue::Empty)
}