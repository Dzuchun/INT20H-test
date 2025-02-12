#[cfg(test)]
mod test {
    use markdown::Block;

    #[test]
    fn test_parse() {
        let input = r#"
# Quest title
## Quest description

--- 
 
efwefwe -- simple text


<question/>
- sds d
+ sd sd
- dsdsdsd 
- s sds

<question/>
<opened />

<img src="img url" />
<video src="img url" controls />

```python
x = 5
print(x * 2)
```

<question/>

<question/>
<img src="href" />
32 -- left
23 -- top
5 -- width
7 -- height
 

42"#;

        println!("{:#?}", markdown::tokenize(input));
    }

    #[test]
    fn test_gen() {
        let tokens = vec![Block::Hr];

        println!("---\n{}\n---", markdown::generate_markdown(tokens));
    }
}

const s: &str = r#"
Question?

<img src="href" />

<question>
- a
+ b
- c
- d
</question>

<question-opened>
(correct answer)
</question-opened>

<question-image>
<img src="href" />
32
23
5
7
</question-image>

<question-image>
<img src="href" />
35
26
</question-image>
"#;

const text: &str = "Quetion?";

const quetion: &str = r"
- a
- b
- c
- d
";
