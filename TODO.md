# TODO

- Change config format to support strings for timeout
  - `default`, `never`, or a millisecond value
- Remove duplicate parsing of values
  - Config file does it and arg parsing
- Rename `summary` to `title`
- Rename `body` to `message`
- `Args` unit tests
- Allow user to provide custom format for title and message
- Improved error handling
  - note the application in the error messages
  - `notify-complete: error goes here`
- Update notifications with progress
- DRY up config tests
