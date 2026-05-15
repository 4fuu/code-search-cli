; Pattern 0: Class declaration
(class_declaration
  name: (identifier) @name) @definition

; Pattern 1: Interface declaration
(interface_declaration
  name: (identifier) @name) @definition

; Pattern 2: Enum declaration
(enum_declaration
  name: (identifier) @name) @definition

; Pattern 3: Record declaration
(record_declaration
  name: (identifier) @name) @definition

; Pattern 4: Method declaration
(method_declaration
  name: (identifier) @name) @definition

; Pattern 5: Constructor declaration
(constructor_declaration
  name: (identifier) @name) @definition

; Pattern 6: Field declaration
(field_declaration
  declarator: (variable_declarator
    name: (identifier) @name)) @definition
