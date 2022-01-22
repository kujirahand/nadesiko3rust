use crate::node::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct NodeContext {
    pub callstack_level: usize,
    pub labels: HashMap<String, Node>,
    pub scopes: Vec<NodeScope>,
    pub files: Vec<String>,
    pub sysfuncs: Vec<SysFuncInfo>,
    errors: Vec<NodeError>,
    error_count: usize,
}

impl NodeContext {
    pub fn new() -> Self {
        // generate system scope and user global scope
        let sys_scope = NodeScope::new();
        let user_global = NodeScope::new();
        let scopes = vec![sys_scope, user_global];
        NodeContext {
            callstack_level: 0,
            labels: HashMap::new(),
            scopes,
            files: vec![],
            sysfuncs: vec![],
            errors: vec![],
            error_count: 0,
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
        // 末端から変数名を検索
        let mut level: isize = (self.scopes.len() - 1) as isize;
        while level >= 0 {
            let scope = &self.scopes[level as usize];
            match scope.get_var_no(name) {
                Some(no) => return Some(NodeVarInfo{name: None, level: level as usize, no:*no}),
                None => {
                    level -= 1;
                    continue;
                }
            }
        }
        None
    }
    pub fn get_var_value(&self, i: &NodeVarInfo) -> Option<NodeValue> {
        if i.level >= self.scopes.len() {
            return None;
        }
        let scope = &self.scopes[i.level];
        Some(scope.var_values[i.no].clone())
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
        let scope = &mut self.scopes[0];
        let no = scope.set_var(name, NodeValue::SysFunc(sys_no, vec![]));
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

type SysArg = Vec<String>;
pub fn sysargs(args: &[&[&str]]) -> Vec<SysArg> {
    let mut result = vec![];
    for arg in args.iter() {
        let mut arg_res: SysArg = vec![];
        for a in arg.iter() {
            arg_res.push(String::from(*a));
        }
        result.push(arg_res);
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
