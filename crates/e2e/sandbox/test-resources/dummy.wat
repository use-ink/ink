;; Dummy contract emitting a dummy event.
(module
	(import "seal0" "seal_deposit_event" (func $seal_deposit_event (param i32 i32 i32 i32)))
	(import "seal0" "seal_return" (func $seal_return (param i32 i32 i32)))
	(import "env" "memory" (memory 1 1))

	(func (export "deploy"))

	(func (export "call")
		;; emit dummy event
    (call $seal_deposit_event
      (i32.const 0) ;; The topics buffer
      (i32.const 0) ;; The topics buffer's length
      (i32.const 8) ;; The data buffer
      (i32.const 4) ;; The data buffer's length
    )

		;; exit with success
		(call $seal_return
			(i32.const 0)	;; flags
			(i32.const 0)	;; returned value
			(i32.const 4)	;; length of returned value
		)
	)
)
