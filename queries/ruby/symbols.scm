; Pattern 0: Module declaration
[
  (module
    name: (constant) @name) @definition
  (module
    name: (scope_resolution
      name: (_) @name)) @definition
]

; Pattern 1: Class declaration
[
  (class
    name: (constant) @name) @definition
  (class
    name: (scope_resolution
      name: (_) @name)) @definition
]

; Pattern 2: Instance method
(method
  name: (identifier) @name) @definition

; Pattern 3: Singleton method
(singleton_method
  name: (identifier) @name) @definition

; Pattern 4: Constant assignment
(assignment
  left: (constant) @name) @definition

; Pattern 5: Variable assignment
(assignment
  left: (identifier) @name) @definition
