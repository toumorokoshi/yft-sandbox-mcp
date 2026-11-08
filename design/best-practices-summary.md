# Best Practices Summary

1. Always use a logging library for printing output to the console.
2. Use third party libraries where possible.
3. Prefer pure functional code. State modification should be in parent, or
   helper methods where they can be well encapsulated and tested.
4. After the code is functional, review the code for any duplication. Refactor
   to re-use via modular utility methods.
5. Always add unit tests.
   - If the file is a script, just add a simple test harness as a sibling, with the suffix "\_test" in the filename.
