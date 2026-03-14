(require "core")

;;@doc
;; Get the ID of the closed document
;; @param crate If it's a workspace, define the crate to search against
(define (get-edition crate)
  (let* ([content (file->string (car (read-dir ".")))]
         [toml    (parse-toml content)]
         [pkg     (hash-ref toml "package")]
         [edition (hash-ref pkg "edition")])
    edition))
