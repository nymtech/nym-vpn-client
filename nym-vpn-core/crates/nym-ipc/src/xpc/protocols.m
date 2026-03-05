#import <Foundation/Foundation.h>

@protocol NSConnectionInterface
- (void)write:(NSData *)buf;
@end

// Anchor class: forces the compiler to emit ObjC metadata that references the protocol
__attribute__((objc_runtime_name("NymXpcProtocolAnchor")))
@interface NymXpcProtocolAnchor : NSObject <NSConnectionInterface>
@end

@implementation NymXpcProtocolAnchor
- (void)write:(NSData *)buf { (void)buf; }
@end

// Force the object file to be linked and ensure protocol reference exists
__attribute__((used))
void nym_force_link_xpc_protocols(void) {
    // This makes a direct reference to the Protocol* in the object file.
    (void)@protocol(NSConnectionInterface);
}