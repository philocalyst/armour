(require "core")

(define (get-edition)
  (hash-try-get (hash-try-get (parse-toml (file->string (car (read-dir ".")))) "package") "edition"))
