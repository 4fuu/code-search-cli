; Pattern 0: Function definition (top-level)
(function_definition
  name: (identifier) @name) @definition

; Pattern 1: Class definition
(class_definition
  name: (identifier) @name) @definition

; Pattern 2: Method (function inside class)
(class_definition
  body: (block
    (function_definition
      name: (identifier) @name) @definition))

; Pattern 3: Module-level assignment
(module
  (expression_statement
    (assignment
      left: (identifier) @name) @definition))
