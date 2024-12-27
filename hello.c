int main() {
  int a = 1, sum = 0;
  {
    a = a + 2; //3
    int b = a + 3; //6
    b = b + 4; //10
    sum = sum + a + b; //13
    {
      b = b + 5; // 15
      int c = b + 6; // 21
      a = a + c; //24
      sum = sum + a + b + c; // 13 + 24 + 15 + 21 = 28 + 45 = 73
      {
        b = b + a; // 15 + 21 = 36
        int a = c + 7; // a = 28
        a = a + 8; // a = 36
        sum = sum + a + b + c; // 73 + 36 + 36 + 21 = 145 + 21 = 166
        {
          b = b + a;
          int b = c + 9;
          a = a + 10;
          const int a = 11;
          b = b + 12;
          sum = sum + a + b + c;
          {
            c = c + b;
            int c = b + 13;
            c = c + a;
            sum = sum + a + b + c;
          }
          sum = sum - c;
        }
        sum = sum - b; // 166 - 36 = 130
      }
      sum = sum - a; // 130 - 24 = 106
    }
  }
  return sum % 77; // 106 - 77 = 2
}