(require "core")

;;@doc
;; Get the ID of the closed document
;; @param cratee If it's a workspace, define the crate to search against
(define (get-edition)
  (let* ([content (file->string (car (read-dir ".")))]
         [toml    (parse-toml content)]
         [pkg     (hash-ref toml "package")]
         [edition (hash-ref pkg "edition")])
    (make-entry "EDITION" edition)))
