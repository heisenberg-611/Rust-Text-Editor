pub struct Syntax {
    pub file_type: &'static str,
    pub file_extensions: &'static [&'static str],
    pub keywords: &'static [&'static str],
    pub single_line_comment: &'static str,
}

impl Default for Syntax {
    fn default() -> Self {
        Syntax {
            file_type: "Text",
            file_extensions: &[],
            keywords: &[],
            single_line_comment: "",
        }
    }
}

impl Syntax {
    pub fn select(filename: &str) -> &'static Syntax {
        let parts: Vec<&str> = filename.split('.').collect();
        if let Some(ext) = parts.last() {
            for syntax in SYNTAX_LIST.iter() {
                if syntax.file_extensions.contains(ext) {
                    return syntax;
                }
            }
        }
        &DEFAULT_SYNTAX
    }

    pub fn default_ref() -> &'static Syntax {
        &DEFAULT_SYNTAX
    }
}

static DEFAULT_SYNTAX: Syntax = Syntax {
    file_type: "Text",
    file_extensions: &[],
    keywords: &[],
    single_line_comment: "",
};

static SYNTAX_LIST: &[Syntax] = &[
    Syntax {
        file_type: "Rust",
        file_extensions: &["rs"],
        keywords: &[
            "fn", "let", "mut", "pub", "use", "mod", "struct", "enum", "impl", "trait", "match",
            "if", "else", "for", "while", "loop", "return", "break", "continue", "const", "static",
            "type", "as", "ref", "in", "where", "crate", "super", "self", "Self", "true", "false",
            "None", "Some", "Ok", "Err",
        ],
        single_line_comment: "//",
    },
    Syntax {
        file_type: "C",
        file_extensions: &["c", "h"],
        keywords: &[
            "switch", "if", "while", "for", "break", "continue", "return", "else", "struct",
            "union", "typedef", "static", "enum", "class", "case", "int", "long", "double",
            "float", "char", "unsigned", "signed", "void", "NULL",
        ],
        single_line_comment: "//",
    },
    Syntax {
        file_type: "C++",
        file_extensions: &["cpp", "hpp", "cc", "cxx", "hh"],
        keywords: &[
            "switch",
            "if",
            "while",
            "for",
            "break",
            "continue",
            "return",
            "else",
            "struct",
            "union",
            "typedef",
            "static",
            "enum",
            "class",
            "case",
            "public",
            "private",
            "protected",
            "friend",
            "inline",
            "virtual",
            "template",
            "using",
            "namespace",
            "true",
            "false",
            "int",
            "long",
            "double",
            "float",
            "char",
            "unsigned",
            "signed",
            "void",
            "NULL",
        ],
        single_line_comment: "//",
    },
    Syntax {
        file_type: "Java",
        file_extensions: &["java"],
        keywords: &[
            "class",
            "public",
            "private",
            "protected",
            "static",
            "final",
            "void",
            "return",
            "if",
            "else",
            "for",
            "while",
            "do",
            "break",
            "continue",
            "switch",
            "case",
            "default",
            "try",
            "catch",
            "finally",
            "import",
            "package",
            "new",
            "this",
            "super",
            "int",
            "double",
            "float",
            "boolean",
            "char",
            "true",
            "false",
            "null",
        ],
        single_line_comment: "//",
    },
    Syntax {
        file_type: "JavaScript",
        file_extensions: &["js", "jsx", "ts", "tsx"],
        keywords: &[
            "function",
            "let",
            "var",
            "const",
            "if",
            "else",
            "for",
            "while",
            "do",
            "return",
            "break",
            "continue",
            "switch",
            "case",
            "default",
            "try",
            "catch",
            "finally",
            "class",
            "extends",
            "new",
            "this",
            "import",
            "export",
            "from",
            "async",
            "await",
            "true",
            "false",
            "null",
            "undefined",
        ],
        single_line_comment: "//",
    },
    Syntax {
        file_type: "Python",
        file_extensions: &["py"],
        keywords: &[
            "def", "class", "if", "elif", "else", "for", "while", "break", "continue", "return",
            "import", "from", "as", "pass", "try", "except", "finally", "raise", "with", "lambda",
            "global", "nonlocal", "True", "False", "None", "and", "or", "not", "is", "in",
        ],
        single_line_comment: "#",
    },
];
