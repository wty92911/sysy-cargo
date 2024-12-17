cargo build && docker run -it --rm -v  ./:/root/compiler maxxing/compiler-dev \
  autotest -koopa -s lv1 /root/compiler