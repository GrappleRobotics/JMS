#include <mbed.h>
#include <EthernetInterface.h>
#include <FlashIAP.h>

DigitalOut led(PB_0);

int main() {
  char buf[256];

  EthernetInterface eth;
  eth.connect();

  SocketAddress a;
  eth.get_ip_address(&a);
  printf("Server IP Address is: %s\n", a.get_ip_address());

  TCPSocket srv, *client;
  srv.open(&eth);
  
  srv.bind(9999);
  
  srv.listen(1);

  while (1) {
    client = srv.accept();
    while (client != nullptr && client->recv(buf, 256) > 0) {
      led = (atoi(buf) != 0);
    }
    ThisThread::sleep_for(50ms);
  }
}

// int main() {
//   // Note: https://www.st.com/resource/en/reference_manual/dm00314099-stm32h742-stm32h743-753-and-stm32h750-value-line-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf
//   // We use Sector 7 of Bank 2 - 0x081E 0000 + 128K

//   const uint32_t CFG_REGION = 0x081E0000;

//   FlashIAP flash;
//   flash.init();

//   char buf1[50] = {0};
//   char buf2[50] = {0};
//   strcpy(buf2, "Hello World ABCD!");

//   int r = flash.read(buf1, CFG_REGION, 50);
//   printf("Read %d - %s\n", r, buf1);

//   ThisThread::sleep_for(100ms);

//   r = flash.program(buf2, CFG_REGION, 50);
//   printf("Programmed %d - %s\n", r, buf2);

//   ThisThread::sleep_for(100ms);

//   r = flash.read(buf1, CFG_REGION, 50);
//   printf("Read %d - %s\n", r, buf1);

//   flash.deinit();

//   while(1) {
//     ThisThread::sleep_for(1s);
//   }
// }