(require "core")

;;@doc
;; Get the ID of the closed document
(define (get-edition)
  (let* ([content (file->string (car (read-dir ".")))]
         [toml    (parse-toml content)]
         [pkg     (hash-ref toml "package")]
         [edition (hash-ref pkg "edition")])
    edition))
