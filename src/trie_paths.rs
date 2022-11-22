pub fn into_index(c: char) -> usize {
  (c as usize) - ('a' as usize)
}

pub fn from_index(i: usize) -> char {
  (i + ('a' as usize)) as u8 as char
}

fn range_bits(s: usize, e: usize) -> u32 {
  (1 << e+1) - (1 << s)
}

fn index_bits(s: char, e: char) -> u32 {
  range_bits(into_index(s), into_index(e))
}

pub fn split_path(path: &str) -> (&str, &str) {
  let (head, tail) = path.split_at(1);
  if head != "[" { return (head, tail) }
  tail.split_once("]").unwrap_or((tail, ""))
}

pub fn char_set_from_group(
  group: &str
) -> u32 {
  let mut prev = 'a';
  let mut chars = group.chars();
  let mut char_set: u32 = 0;

  while let Some(curr) = chars.next() {
    if curr == '-' {
      let next = chars.next()
        .filter(|c| c.is_ascii_alphabetic())
        .unwrap_or('z');
      char_set |= index_bits(prev, next);
      prev = next
    } else if curr.is_ascii_alphabetic() {
      char_set |= index_bits(curr, curr);
      prev = curr
    } else {
      char_set = (1 << 26) - 1;
    }
  }

  char_set
}
