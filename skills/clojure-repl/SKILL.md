---
name: clojure-repl
description: REPL-Driven Development Workflow in Clojure
---

# REPL-Driven Development Workflow in Clojure

## 1. Check if REPL is running

Run testing expression in the begining of a session JUST ONCE
```clojure
brepl -e '(+ 1 2)'
3
```

If REPL is not running, that do not use this skill at all. 
Run tests via command line with `bb test`, and clojure code if needed via `clj -e`

## 2. Run tests in REPL

- In a Clojure project, after changes run all tests at once in REPL with: `brepl -e '(user/run-tests)'`
- For small changes or isolated changes run subset of tests: 
  - run a specific test: `brepl -e "(user/run-tests 'myproject.search-queries-test/test-preprocess-search-query-integration)"` 
  - or a specific namespace: `brepl -e '(user/run-tests "test/myproject/home_test.clj")`

## 3. Explore and Experiment

Start by exploring small pieces of functionality in isolation:

```clojure
; Try out simple expressions
brepl -e '(+ 1 2 3)'
6

; Explore and define data structures
brepl -e '(def sample-data [{:name "Alice" :score 42} {:name "Bob" :score 27} {:name "Charlie" :score 35}])'
#'user/sample-data

; REPL keeps state and definitions between your commands
; Try simple transformations
brepl -e '(map :score sample-data)'
(42 27 35)
```