pub mod replace_mod {
    use bytes::Bytes;
    use std::collections::VecDeque;
    use std::pin::Pin;
    use std::sync::mpsc;
    use tokio_util::io::ReaderStream;
    use std::sync::Arc;

    pub struct ReplaceStream<T: tokio_stream::Stream + std::marker::Unpin> {
        stream: T,
        pub buffer: VecDeque<u8>,
        pub triggers: Vec<(Vec<u8>, Arc<dyn Fn(&mut ReplaceStream<T>) -> () + Send + Sync>)>,
    }

    pub fn replacment_builder<'a, T: tokio::io::AsyncRead + std::marker::Unpin>(
        stream: T,
        triggers: Vec<(
            Vec<u8>,
            Arc<dyn Fn(&mut ReplaceStream<ReaderStream<T>>) -> () + Send + Sync>,
        )>,
    ) -> ReplaceStream<ReaderStream<T>> {
        ReplaceStream {
            stream: ReaderStream::new(stream),
            buffer: VecDeque::new(),
            triggers,
        }
    }

    impl<T: tokio_stream::Stream<Item = Result<Bytes, std::io::Error>> + std::marker::Unpin>
        tokio_stream::Stream for ReplaceStream<T>
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

            //print!("triggers len: {}    ", loc_triggers.len());
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

                //println!("Read: {:02x?}, matches: {}", read, loc_triggers.len());

                if matches.is_some() || loc_triggers.is_empty() {
                    break;
                }
            }
            
            add_to_buffer.append(&mut sref.buffer);
            sref.buffer = add_to_buffer;

            //This bit is dumb but I can't see how else to please the borrow checker
            let mut c = None;
            if let Some(&(_a, b)) = matches {
                c = Some(b.clone());
            }

            match c {
                Some(a) => (a)(sref),
                None => {},
            }

            //if let c = Some(a) {
            //    (a)(sref);
            //}
             

            
            //let res = sref.buffer.pop_front();
            std::task::Poll::Ready(sref.buffer.pop_front())
        }
    }

    impl<T: tokio_stream::Stream<Item = Result<Bytes, std::io::Error>> + std::marker::Unpin>
        ReplaceStream<T>
    {
        pub fn replace(&mut self, target: &[u8], repl: &[u8]) {
            self.buffer.drain(0..target.len());
            let mut new = VecDeque::from(repl.to_vec());
            new.append(&mut self.buffer);
            self.buffer = new;
        }

        pub fn replace_vec(&mut self, target: &Vec<u8>, repl: &Vec<u8>) {
            println!("replacment triggered");
            self.buffer.drain(0..target.len());
            let mut new = VecDeque::from(repl.clone());
            new.append(&mut self.buffer);
            self.buffer = new;
        }

        pub fn message<S : Copy + Send + Sync + 'static>(sender: mpsc::SyncSender<S>, message: S) -> Arc<dyn Fn(&mut ReplaceStream<T>) -> () + Send + Sync> {
            println!("message");
            //sender.send(message);
            Arc::new(move |x| {sender.send(message);})
        }


        pub fn rpl_boxed (target: Vec<u8>, repl: Vec<u8>) -> Arc<dyn Fn(&mut ReplaceStream<T>) -> () + Send + Sync> {
            //Box::new(move |x| x.replace_vec(&target, &repl))
            Arc::new(move|x| x.replace_vec(&target, &repl))

        }
    }
}
