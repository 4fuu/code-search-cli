; Pattern 0: Function declaration
(function_declaration
  name: (identifier) @name) @definition

; Pattern 1: Class declaration
(class_declaration
  name: (type_identifier) @name) @definition

; Pattern 2: Interface declaration
(interface_declaration
  name: (type_identifier) @name) @definition

; Pattern 3: Type alias
(type_alias_declaration
  name: (type_identifier) @name) @definition

; Pattern 4: Enum declaration
(enum_declaration
  name: (identifier) @name) @definition

; Pattern 5: Method definition
(method_definition
  name: (property_identifier) @name) @definition

; Pattern 6: Variable (export)
(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name) @definition))

; Pattern 7: Variable (top-level)
(program
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name) @definition))
