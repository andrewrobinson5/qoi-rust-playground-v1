# Summary
A little weekend project to implement the Quite OK Image Format specification in Rust with just the spec sheet, prior knowledge, and a pen & pad.

DON'T USE THIS IN YOUR PROJECTS, THIS IS THE FIRST DRAFT OF MY FIRST RUST PROJECT AND IS HORRIBLY UNOPTIMIZED AND UNUSABLE.

# Impressions
This format was cool to figure out. The [spec sheet](https://qoiformat.org/qoi-specification.pdf) was very short and simple, and while the information provided was sufficient, some of the information I went looking for was 'compressed' into other pieces of information. I found myself needing to read through it multiple times to be able to process what the document said into what I needed. I spent more time understanding the format than writing the code. This made it a nice puzzle. I recommend printing out the spec sheet, going offline, and implementing QOI <=> RGBA off the top of your head in your favorite language as a fun exercise.

As I'm still relatively new to Rust, I know this could have been much more idiomatic, and I will be looking into how popular file parsers are implemented in Rust to learn some new verbiage that I can apply for the V2. I'll also take a look at how other libraries structure themselves and plan their APIs so that the next iteration may be useful, rather than just a toy.

My code layout makes no sense. This is because the program began as a simple (de)serializer for QOI files, and expanded to an encoder/decoder once that was done. In V2, I will give more attention to making sure the inputs/outputs are more useful, and I'll streamline the functions to directly convert between the formats instead of first bringing them into an intermediary type-friendly object.

All things considered, I'm pretty happy with this project. I would recommend any programmer try to make the same thing. Now with a new understanding of some pain points, I'm better equipped to structure my learning moving forward. Cheers!
