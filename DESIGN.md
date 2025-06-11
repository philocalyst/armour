# Lua Module structure

All you need to do is define an object, named arbitraily (although the standard would be to call it M, for module), and define a method ontop of it called "build_badge" -- this is what is responsible for handling all of the logic related to the creation of a badge. Think of it as your main entry point.

You'll be taking a shallow input table that you define the types of using doc comments on this build badge object method. This is responsible for the entire API generation!! And how the rust side will call your code!! Very important!! Please test ahead of time if you can.

