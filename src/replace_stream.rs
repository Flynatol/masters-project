pub mod replace_mod {
    use bytes::Bytes;
    use std::collections::VecDeque;
    use std::pin::Pin;

    pub struct ReplaceStream2<'a, T: tokio_stream::Stream + std::marker::Unpin> {
        stream: T,
        buffer: VecDeque<u8>,
        triggers: Vec<(&'a [u8], fn(&mut ReplaceStream2<T>) -> ())>,
    }

    pub fn replacment_builder<'a, T: tokio_stream::Stream + std::marker::Unpin>(
        stream: T,
        triggers: Vec<(&'a [u8], fn(&mut ReplaceStream2<T>) -> ())>,
    ) -> ReplaceStream2<T> {
        ReplaceStream2 {
            stream,
            buffer: VecDeque::new(),
            triggers,
        }
    }

    impl<T: tokio_stream::Stream<Item = Result<Bytes, std::io::Error>> + std::marker::Unpin>
        tokio_stream::Stream for ReplaceStream2<'_, T>
    {
        type Item = u8;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<u8>> {
            let mut add_to_buffer: VecDeque<u8> = VecDeque::new();
            let sref = self.get_mut();
            let mut matches;

            let mut loc_triggers = sref
                .triggers
                .iter()
                .map(|(a, b)| (&a[..], b))
                .collect::<Vec<_>>();

            loop {
                let read = match sref.buffer.is_empty().to_owned() {
                    true => {
                        match Pin::new(&mut sref.stream).poll_next(cx) {
                            std::task::Poll::Ready(result) => {
                                match result {
                                    Some(b) => {
                                        //write bytes into buffer
                                        let mut bytes = b
                                            .expect("Some contains error")
                                            .iter()
                                            .map(|&f| f)
                                            .collect::<VecDeque<u8>>();
                                        sref.buffer.append(&mut bytes);
                                        sref.buffer.pop_front().unwrap()
                                    }
                                    None => {
                                        //We have reached EOF
                                        add_to_buffer.append(&mut sref.buffer);
                                        sref.buffer = add_to_buffer;
                                        println!("Disconnected, stream retuned EOF");
                                        return std::task::Poll::Ready(None);
                                    }
                                }
                            }
                            std::task::Poll::Pending => {
                                //We're waiting on the underlying stream
                                add_to_buffer.append(&mut sref.buffer);
                                sref.buffer = add_to_buffer;
                                return std::task::Poll::Pending;
                            }
                        }
                    }
                    false => {
                        //If buffer is available read from that instead
                        sref.buffer.pop_front().unwrap()
                    }
                };

                add_to_buffer.push_back(read);

                loc_triggers = loc_triggers
                    .into_iter()
                    .filter(|(x, _)| x.first() == Some(&read))
                    .map(|(a, b)| (&a[1..], b))
                    .collect::<Vec<_>>();

                matches = loc_triggers.iter().filter(|(x, _)| x.is_empty()).next();

                if matches.is_some() || loc_triggers.is_empty() {
                    break;
                }
            }

            add_to_buffer.append(&mut sref.buffer);
            sref.buffer = add_to_buffer;

            if let Some((_a, &b)) = matches {
                (b)(sref);
            }

            std::task::Poll::Ready(sref.buffer.pop_front())
        }
    }

    impl<T: tokio_stream::Stream<Item = Result<Bytes, std::io::Error>> + std::marker::Unpin>
        ReplaceStream2<'_, T>
    {
        fn replace(&mut self, target: &[u8], repl: &[u8]) {
            self.buffer.drain(0..target.len());
            let mut new = VecDeque::from(repl.to_vec());
            new.append(&mut self.buffer);
            self.buffer = new;
        }
    }
}
