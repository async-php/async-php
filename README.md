src/http.rs写得太烂了，帮我重构一下,有一些已经写了一点，但大概率是错误的，你可以大刀阔斧的改造。主要是这些问题：
* 模仿golang，抽象出io.Reader/io.ReadCloser/io.ReaderAt/io.ReadSeek/io.ReaderFrom等，各种Writer同理。PHP层也要有对等概念
* io的接口需要写在rust中
* 把原先的http.rs和http_client.rs删除，逻辑搬到src/http中
* 无论是request/response, body应该是一个流，实现上一点提到的各种接口，使用AsyncRead/AsyncWrite/AsyncSeek等去做。最好暴露接口能让php端去适配实现，你提供默认的。
* request里面的response指针，应该是请求后绑定上的，而不是一直有的。
* http3需要支持下，无论是http1/http1.1/http2/http3都需要支持在php侧传递证书而不是自签名。当然，这是一个给用户的可选项。
* rust调整后相关的php逻辑记得也跟着改一下
* 编译要通过，测试代码要通过

要求：
1. rust层只负责最简内核，尽量php完成大部分工作；
2. 不能降低当前rust和ext-php-rs版本
3. Php的类在\Async下，封装\Async\Kernel下的类；
4. Php的类也需要按照功能模块整理好，不能都在\Async下，最好跟着rust能够匹配；
5. Php如果需要hook，尽量通过php启动时设置disable_functions/disable_classes然后定义用户层方法或者类的方式实现.