; Pattern 0: Function declaration
(function_declaration
  name: (identifier) @name) @definition

; Pattern 1: Method declaration
(method_declaration
  name: (field_identifier) @name) @definition

; Pattern 2: Type declaration (all types: struct, interface, alias, etc.)
; Specific kind is determined in Go lang module by inspecting the type child.
(type_declaration
  (type_spec
    name: (type_identifier) @name) @definition)

; Pattern 3: Const
(const_declaration
  (const_spec
    name: (identifier) @name) @definition)

; Pattern 4: Var
(var_declaration
  (var_spec
    name: (identifier) @name) @definition)
