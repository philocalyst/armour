(provide my-fun)
;;@doc
;; Get the ID of the closed document
(define (my-fun x . args)
  (displayln "called with" (length args) "optional args")
  (displayln "optional args:" args))

(provide file->string)
(define (file->string path)
  (let ([file (open-input-file path)]) (read-port-to-string file)))
