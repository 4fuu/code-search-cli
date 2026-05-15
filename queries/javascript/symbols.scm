; Pattern 0: Function declaration
(function_declaration
  name: (identifier) @name) @definition

; Pattern 1: Class declaration
(class_declaration
  name: (identifier) @name) @definition

; Pattern 2: Method definition
(method_definition
  name: (property_identifier) @name) @definition

; Pattern 3: Function-valued variable
[
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name
      value: [(arrow_function) (function_expression)]) @definition)
  (variable_declaration
    (variable_declarator
      name: (identifier) @name
      value: [(arrow_function) (function_expression)]) @definition)
]

; Pattern 4: Exported variable/const
(export_statement
  declaration: [
    (lexical_declaration
      (variable_declarator
        name: (identifier) @name) @definition)
    (variable_declaration
      (variable_declarator
        name: (identifier) @name) @definition)
  ])

; Pattern 5: Top-level variable/const
(program
  [
    (lexical_declaration
      (variable_declarator
        name: (identifier) @name) @definition)
    (variable_declaration
      (variable_declarator
        name: (identifier) @name) @definition)
  ])
