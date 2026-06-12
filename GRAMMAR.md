# B-minor Grammar

Note 1: whitespace (space, tab, linefeed, carriage return) and comments are the
same as in C and C++.

Note 2: The following are keywords and can't be used as identifiers: `array`,
`boolean`, `char`, `else`, `false`, `for`, `function`, `if`, `integer`,
`print`, `return`, `string`, `true`, `void`.

Note 3: characters and strings are _not_ wide; they contain simply ASCII characters,
as far as the language is concerned.

Note 4: the only special quoted characters are `\n` and `\0`. Any other character
preceded by a backslash (`\`) will became that character itself - this is of course
useful mostly when including double quotes into a string...

Note 5: The `if` condition expression **must** evaluate to a boolean.

Note 6: Array size in declarations must be a constant value (a literal in this case
for global declarations. Within blocks though, this can be any expression that evaluates
to an integer - this will be enforced during the semantic phase though.

Note 7: The expressions are not included in the grammar - use your imagination :-P


    program               : [ declaration , { declaration } ] ;

    declaration           : function_declaration | var_declaration ;

    statement             : assignment_statement
                          | for_statement
                          | if_statement
                          | print_statement
                          | return_statement
                          | function_call
                          | block
                          ;

    block                 : '{' , { decl_or_statement } , '}' ;

    decl_or_statement     : var_declaration
                          | array_declaration
                          | statement
                          ;

    assignment_statement  : assignment , ';'

    for_statement         : 'for' , '(' , [ assignment , { ',' , assignment } ] , ';' , expression , ';' , expression , ')' , block ;

    assignment            : identifier , '=' , expression ;

    if_statement          : 'if' , '(' , expression , ')' , block , [ 'else' , block ] ;

    print_statement       : 'print' , [ expression , { ',' , expression } ] , ';' ;

    return_statement      : 'return' , expression ;

    function_call         : identifier , '(' , [ expr , { ',' , expr } ] , ')' , ';' ;

    function_declaration  : function_signature , '=' , block ;

    function_prototype    : function_signature , ';' ;

    function_signature    : identifier , ':' , 'function' , return_type , '(' , [ parameter , { ',' , parameter } ] , ')' ;

    return_type           : type | 'void' ;

    parameter             : identifier , ':' , ( type | array_type ) ;

    array_type            : array_type_pref , { array_type_pref } , type ;

    array_type_pref       : 'array' , '[' , ']' ;

    var_declaration       : identifier , ':' ( scalar_declaration | array_declaration ) , ';' ;

    array_declaration     : array_decl_pref , { array_decl_pref } , type ,  [ array_initialization ] ;

    array_decl_pref       : 'array' , '[' , expression , ']' ;

    array_initialization  : '{' , array_init_element , { ',' , array_init_element } , '}' ;

    array_init_element    : literal
                          | array_initialization
                          ;

    scalar_declaration    : type , [ var_initialization ] , ';' ;

    scalar_initialization : '=' , literal ;

    type                  : 'boolean' | 'char' | 'integer' | 'string' ;

    identifier            : ( letter | '_' ) , { letter | digit | '_' } ;

    integer_literal       : digit , { digit } ;

    boolean_literal       : 'true' | 'false' ;

    char_literal          : "'" , 8_BIT_ASCII_CHARACTER , "'" ;

    string_literal        : '"' , [ STRING_OF_ASCII_CHARACTERS ] , '"' ;

    digit                 : '0' | ... | '9' ;
    letter                : 'A' | ... | 'Z' | 'a' | ... | 'z' ;

