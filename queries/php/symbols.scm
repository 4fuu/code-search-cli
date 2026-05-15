; Pattern 0: Namespace definition
(namespace_definition
  name: (namespace_name) @name) @definition

; Pattern 1: Interface declaration
(interface_declaration
  name: (name) @name) @definition

; Pattern 2: Trait declaration
(trait_declaration
  name: (name) @name) @definition

; Pattern 3: Class declaration
(class_declaration
  name: (name) @name) @definition

; Pattern 4: Function definition
(function_definition
  name: (name) @name) @definition

; Pattern 5: Method declaration
(method_declaration
  name: (name) @name) @definition

; Pattern 6: Constant declaration
(const_declaration
  (const_element
    (name) @name)) @definition

; Pattern 7: Property declaration
(property_declaration
  (property_element
    name: (variable_name
      (name) @name))) @definition
