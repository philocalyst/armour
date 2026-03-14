(provide my-fun)
(define (my-fun x . args)
  (displayln "called with" (length args) "optional args")
  (displayln "optional args:" args))
