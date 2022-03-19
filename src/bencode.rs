// bencode.rs
// Copyright 2022 Matti Hänninen
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("end of input reached before completing next bencode object")]
    UnexpectedEnd,
    #[error("malformed bencode input")]
    BadInput,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjType {
    ByteString,
    Dictionary,
    Integer,
    List,
}

pub fn scan_next(bytes: &[u8]) -> Result<(ObjType, usize), Error> {
    let res = scan_next_iter(bytes.iter().enumerate().peekable())?;
    Ok((res.0, res.1))
}

fn scan_next_iter<'a, I>(
    mut iter: Peekable<I>,
) -> Result<(ObjType, usize, Peekable<I>), Error>
where
    I: 'a + Iterator<Item = (usize, &'a u8)>,
{
    let (i_first, b_first) = iter.next().ok_or(Error::UnexpectedEnd)?;
    match b_first {
        // Integer
        b'i' => {
            match iter.next().ok_or(Error::UnexpectedEnd)?.1 {
                b'-' => match iter.next().ok_or(Error::UnexpectedEnd)?.1 {
                    b'1'..=b'9' => (),
                    _ => return Err(Error::BadInput),
                },
                b'0' => {
                    let (i, b) = iter.next().ok_or(Error::UnexpectedEnd)?;
                    if *b == b'e' {
                        return Ok((ObjType::Integer, i - i_first + 1, iter));
                    } else {
                        return Err(Error::BadInput);
                    }
                }
                b'1'..=b'9' => (),
                _ => return Err(Error::BadInput),
            };
            while let Some((i, b)) = iter.next() {
                match *b {
                    b'0'..=b'9' => (),
                    b'e' => {
                        return Ok((ObjType::Integer, i - i_first + 1, iter))
                    }
                    _ => return Err(Error::BadInput),
                }
            }
            Err(Error::UnexpectedEnd)
        }
        // List
        b'l' => loop {
            let (i, b) = *iter.peek().ok_or(Error::UnexpectedEnd)?;
            if *b == b'e' {
                iter.next().unwrap();
                return Ok((ObjType::List, i - i_first + 1, iter));
            } else {
                iter = scan_next_iter(iter)?.2;
            }
        },
        // Dictionary
        b'd' => loop {
            let (i, b) = *iter.peek().ok_or(Error::UnexpectedEnd)?;
            if *b == b'e' {
                iter.next().unwrap();
                return Ok((ObjType::Dictionary, i - i_first + 1, iter));
            } else {
                let key = scan_next_iter(iter)?;
                if key.0 != ObjType::ByteString {
                    return Err(Error::BadInput);
                }
                iter = scan_next_iter(key.2)?.2;
            }
        },
        // Byte string
        b'0'..=b'9' => {
            let mut l = (b_first - b'0') as usize;
            loop {
                let (i, b) = iter.next().ok_or(Error::UnexpectedEnd)?;
                match *b {
                    b'0'..=b'9' => {
                        l = 10 * l + (*b - b'0') as usize;
                    }
                    b':' => {
                        if l > 0 {
                            let (i_last, _) =
                                iter.nth(l - 1).ok_or(Error::UnexpectedEnd)?;
                            return Ok((
                                ObjType::ByteString,
                                i_last - i_first + 1,
                                iter,
                            ));
                        } else {
                            return Ok((
                                ObjType::ByteString,
                                i - i_first + 1,
                                iter,
                            ));
                        }
                    }
                    _ => return Err(Error::BadInput),
                }
            }
        }
        _ => Err(Error::BadInput),
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn bad_input() {
        // Just trash
        assert_eq!(scan_next(b"trash"), Err(Error::BadInput));

        // Illegal integer
        assert_eq!(scan_next(b"i+1e"), Err(Error::BadInput));
        assert_eq!(scan_next(b"i-0e"), Err(Error::BadInput));
        assert_eq!(scan_next(b"i01e"), Err(Error::BadInput));
    }

    #[test]
    fn partial_input() {
        // Empty input
        assert_eq!(scan_next(b""), Err(Error::UnexpectedEnd));

        // Partial integer
        assert_eq!(scan_next(b"i12345"), Err(Error::UnexpectedEnd));

        // Partial byte string
        assert_eq!(scan_next(b"1:"), Err(Error::UnexpectedEnd));
        assert_eq!(scan_next(b"4:foo"), Err(Error::UnexpectedEnd));
    }

    #[test]
    fn integer() {
        assert_eq!(scan_next(b"i0e"), Ok((ObjType::Integer, 3)));
        assert_eq!(scan_next(b"i1e"), Ok((ObjType::Integer, 3)));
        assert_eq!(scan_next(b"i-1e"), Ok((ObjType::Integer, 4)));
        assert_eq!(scan_next(b"i12345e"), Ok((ObjType::Integer, 7)));
    }

    #[test]
    fn byte_string() {
        // Apparently this is okay
        assert_eq!(scan_next(b"0:"), Ok((ObjType::ByteString, 2)));
        assert_eq!(scan_next(b"3:foo"), Ok((ObjType::ByteString, 5)));
        // Apparently this is okay
        assert_eq!(scan_next(b"03:foo"), Ok((ObjType::ByteString, 6)));
        assert_eq!(scan_next(b"10:byte_string"), Ok((ObjType::ByteString, 13)));
    }

    #[test]
    fn list() {
        assert_eq!(scan_next(b"le"), Ok((ObjType::List, 2)));
        assert_eq!(scan_next(b"li0ee"), Ok((ObjType::List, 5)));
        assert_eq!(scan_next(b"lli0eee"), Ok((ObjType::List, 7)));
    }

    #[test]
    fn dictionary() {
        assert_eq!(scan_next(b"de"), Ok((ObjType::Dictionary, 2)));
        assert_eq!(scan_next(b"d3:fooi0ee"), Ok((ObjType::Dictionary, 10)));
        assert_eq!(
            scan_next(b"d3:bar0:3:fooi0ee"),
            Ok((ObjType::Dictionary, 17))
        );
    }

    #[test]
    fn can_have_trailing_input() {
        assert_eq!(
            scan_next(b"d3:bar0:3:fooi0ee13:trailing_input"),
            Ok((ObjType::Dictionary, 17))
        );
    }
}
