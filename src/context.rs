use crate::node::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct NodeContext {
    pub callstack_level: usize,
    pub labels: HashMap<String, Node>,
    pub scopes: NodeScopeList,
    pub files: Vec<String>,
    pub sysfuncs: Vec<SysFuncInfo>,
    errors: Vec<NodeError>,
    error_count: usize,
    pub try_break: Option<usize>,
    pub try_continue: Option<usize>,
}

impl NodeContext {
    pub fn new() -> Self {
        NodeContext {
            callstack_level: 0,
            labels: HashMap::new(),
            scopes: NodeScopeList::new(),
            files: vec![],
            sysfuncs: vec![],
            errors: vec![],
            error_count: 0,
            try_break: None,
            try_continue: None,
        }
    }
    // for file management
    pub fn set_filename(&mut self, filename: &str) -> u32 {
        match self.find_files(filename) {
            Some(fileno) => fileno,
            None => {
                let fileno = self.files.len() as u32;
                self.files.push(filename.to_string());
                fileno
            },
        }
    }
    pub fn find_files(&self, filename: &str) -> Option<u32> {
        for (i, fname) in self.files.iter().enumerate() {
            if fname == filename { return Some(i as u32); }
        }
        None
    }
    // for error
    pub fn has_error(&self) -> bool {
        self.error_count > 0
    }
    pub fn get_error_str(&self) -> String {
        let mut res = String::new();
        for e in self.errors.iter() {
            res.push_str(&format!("{}\n", e.to_string()));
        }
        res
    }
    pub fn throw_error(&mut self, kind: NodeErrorKind, level: NodeErrorLevel, msg: String, line: u32, fileno: u32) {
        let err = NodeError::new(kind, level, msg, line, fileno);
        println!("{}", &err.to_string());
        self.errors.push(err);
        match level {
            NodeErrorLevel::Error => self.error_count += 1,
            _ => {},
        }
    }
    // for scope variables
    pub fn find_var_info(&self, name: &str) -> Option<NodeVarInfo> {
        self.scopes.find_var(name)
    }
    pub fn get_var_value(&self, info: &NodeVarInfo) -> Option<NodeValue> {
        self.scopes.get_var_value(info)
    }
    #[allow(dead_code)]
    pub fn get_var_meta(&self, info: &NodeVarInfo) -> Option<NodeVarMeta> {
        self.scopes.get_var_meta(info)
    }
    // add system func
    pub fn add_sysfunc(&mut self, name: &str, args: Vec<SysArg>, func: SysFuncType) -> usize {
        // add func to sysfuncs
        let sys_no = self.sysfuncs.len();
        let sfi = SysFuncInfo{
            func,
            args,
        };
        self.sysfuncs.push(sfi);
        // add name to scope
        let scope = &mut self.scopes.scopes[0];
        let no = scope.set_var(name, NodeValue::SysFunc(String::from(name), sys_no, vec![]));
        scope.var_metas[no].read_only = true;
        scope.var_metas[no].kind = NodeVarKind::SysFunction;
        sys_no     
    }
}

type SysFuncType = fn(&mut NodeContext, Vec<NodeValue>) -> NodeValue;

#[derive(Clone)]
pub struct SysFuncInfo {
    pub func: SysFuncType,
    pub args: Vec<SysArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SysArg {
    pub name: String, 
    pub josi_list: Vec<String>
}

pub fn sysargs(args: &[&[&str]]) -> Vec<SysArg> {
    let mut result = vec![];
    let mut a: char = 'A';
    for arg in args.iter() {
        let mut arg_res: Vec<String> = vec![];
        for a in arg.iter() {
            arg_res.push(String::from(*a));
        }
        let a_name = String::from(a);
        result.push(SysArg{ name: a_name, josi_list: arg_res });
        a = ((a as u8) + 1u8) as char;
    }
    result
}

#[allow(dead_code)]
#[derive(Debug,Clone,Copy)]
pub enum NodeErrorKind {
    ParserError,
    RuntimeError,
}
#[allow(dead_code)]
#[derive(Debug,Clone,Copy)]
pub enum NodeErrorLevel {
    Hint, Warning, Error
}

#[derive(Debug,Clone)]
pub struct NodeError {
    pub kind: NodeErrorKind,
    pub level: NodeErrorLevel,
    pub message: String,
    pub line: u32,
    pub fileno: u32,
}

impl NodeError {
    pub fn new(kind: NodeErrorKind, level: NodeErrorLevel, message: String, line: u32, fileno: u32) -> NodeError {
        Self {
            kind,
            level,
            message,
            line,
            fileno
        }
    }
    pub fn to_string(&self) -> String {
        let kind_str = match self.kind {
            NodeErrorKind::ParserError => "解析時",
            NodeErrorKind::RuntimeError => "実行時"
        };
        let level_str = match self.level {
            NodeErrorLevel::Error => "エラー",
            NodeErrorLevel::Warning => "警告",
            NodeErrorLevel::Hint => "ヒント",
        };
        format!("[{}{}]({}){}", kind_str, level_str, self.line, self.message)
    } 
}
