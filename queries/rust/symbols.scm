; Pattern 0: Function (top-level fn)
(function_item
  name: (identifier) @name) @definition

; Pattern 1: Method (fn inside impl)
(impl_item
  body: (declaration_list
    (function_item
      name: (identifier) @name) @definition))

; Pattern 2: Struct
(struct_item
  name: (type_identifier) @name) @definition

; Pattern 3: Enum
(enum_item
  name: (type_identifier) @name) @definition

; Pattern 4: Trait
(trait_item
  name: (type_identifier) @name) @definition

; Pattern 5: Impl
(impl_item
  type: (_) @name) @definition

; Pattern 6: Type alias
(type_item
  name: (type_identifier) @name) @definition

; Pattern 7: Const
(const_item
  name: (identifier) @name) @definition

; Pattern 8: Static
(static_item
  name: (identifier) @name) @definition

; Pattern 9: Module
(mod_item
  name: (identifier) @name) @definition
