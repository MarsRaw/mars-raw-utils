New stuff:

- async fetching from the server(s), why? because blocking requests is for the 90s, this is powered by Tokio

- error handling with anyhow, the premier library for handling errors in applications (there's another library by the same maintainer for Errors in library code called thiserror)

- simplified the fetch/get functionality to avoid bloat ontop of the reqwest crate -- which has most of what you seem to be after builtin.

- I've swapped a few places where a Result<T> was being returned for an Option<T>, I think this is more idiomatic for things like writing to disk based on whether a file exists on disk or not.

- had to add async_trait for the runnable subcommand.
- had to update tests.

notes:

- runabble submd is nolonger the impl on a few of the calls to XYZ::run, because of the difficulties implementing Send/Sync over a crate boundary and on a Trait (as opposed to an enum/struct where it's easy.) This would mean changing code in the SCII crate, which is, well seperated from this stuff. So the solution I've gone with (that I am not paticularly fond of) is to keep all the methods but not have it `impl RunableSubcommand for XYZ`, but rather just `impl XYZ`. so all the methods are the same, which means no cross-cutting changes all over, but we loose the benefits of it being under the same trait bounds.
- this is a pretty scattergun, and minimal refactor to get the app to be async.
- have left lots of the wrappers you'd made in there purely to prevent downstream breaks in the mru bin.

todo:

- refactor out all the places you're wrapping library code.
- unify the error handling to be uniform across (i.e no more unwraps, anywhere!, using anyhow and ?s)

* bonus to the above, take your constants that you were passing out for errors and use them with anyhow's `with_context(||)` which is a cool closure for you to inject additional error infor in on-the-fly.

- style up the rest of the codebase, i.e there's lots of places we can have beautiful one-liners, this is particularly nice when we're using the `?` operator as it easily allows us to propigate errors up to the call site (which helps anyone looking at your app in future to fix bugs etc)

- there's a fair bit of ITM around, that's gotta go -- where possible create values without resorting to creating things you mutate.

- there's a lot of duplicated code between files and such.. this should be factored out i.e remote.rs which is, named such in few places contains a large ammount of the same code/calls/functions etc.
