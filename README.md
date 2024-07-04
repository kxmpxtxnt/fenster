# [Fenster](https://dasfenster.org)

## About this project

As you can see, our current school newspaper is broken. So about a quarter of a year before the 2024 summer vacation, I
took on the task of rebuilding it from scratch.

## Progress and setbacks

At the beginning I had planned to write the project completely in Kotlin as this was the language I was using for
everything at the time. Since I had also done more with kotlin multiplatform at the time, I also started this project
with multiplatform. Backend as well as frontend. After some time I came across the problem with encryption. There was
simply no reasonable library with which I wanted to implement the desired encryption. So I had the idea to use the
C-compatibility of Kotlin native and let Rust generate the C code. A tool for this already exists. After some
discussions, we came to the conclusion that this effort would not be worthwhile and that the entire project could be
developed in Rust instead, which would be a good idea since I wanted to learn Rust anyway. While we are at it I have to
say as a disclaimer that this is my first Rust project and therefore probably has some problems / gaps. So please don't
use this project for very important things if you plan to do so.

## Structure

As already mentioned, the backend is written in rust. The frontend for first web and later mobile in kotlin. Maybe the
website will be in another language but im unsure.
Communication is done via a rest api which is documented (maybe xd) via openapi.

## Checklist
### Backend:

- [ ] Auth
    - [ ] Login
    - [ ] Logout
    - [ ] Refresh
- [ ] User
    - [ ] Register
    - [ ] Delete
- [ ] Articles
    - [ ] Create
    - [ ] Un/-Publish
    - [ ] Edit
    - [ ] Delete

## Contribution

Feel free to open a clone / pr and change everything you think should be. Any mentions with constructive criticism /
justified feedback is highly appreciated. All other stuff will be ignored.