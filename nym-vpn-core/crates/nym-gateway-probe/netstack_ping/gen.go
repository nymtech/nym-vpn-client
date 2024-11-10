package main

/*
// Generated by rust2go. Please DO NOT edit this C part manually.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct StringRef {
  const uint8_t *ptr;
  uintptr_t len;
} StringRef;

typedef struct ListRef {
  const void *ptr;
  uintptr_t len;
} ListRef;

typedef struct NetstackRequestRef {
  struct StringRef wg_ip;
  struct StringRef private_key;
  struct StringRef public_key;
  struct StringRef endpoint;
  struct StringRef dns;
  struct ListRef ping_hosts;
  struct ListRef ping_ips;
  uint8_t num_ping;
  uint64_t send_timeout_sec;
  uint64_t recv_timeout_sec;
  uint8_t ip_version;
} NetstackRequestRef;

typedef struct NetstackResponseRef {
  bool can_handshake;
  uint16_t sent_ips;
  uint16_t received_ips;
  uint16_t sent_hosts;
  uint16_t received_hosts;
  bool can_resolve_dns;
} NetstackResponseRef;

// hack from: https://stackoverflow.com/a/69904977
__attribute__((weak))
inline void NetstackCall_ping_cb(const void *f_ptr, struct NetstackResponseRef resp, const void *slot) {
((void (*)(struct NetstackResponseRef, const void*))f_ptr)(resp, slot);
}
*/
import "C"
import (
	"runtime"
	"unsafe"
)

var NetstackCallImpl NetstackCall

type NetstackCall interface {
	ping(req NetstackRequest) NetstackResponse
}

//export CNetstackCall_ping
func CNetstackCall_ping(req C.NetstackRequestRef, slot *C.void, cb *C.void) {
	resp := NetstackCallImpl.ping(newNetstackRequest(req))
	resp_ref, buffer := cvt_ref(cntNetstackResponse, refNetstackResponse)(&resp)
	C.NetstackCall_ping_cb(unsafe.Pointer(cb), resp_ref, unsafe.Pointer(slot))
	runtime.KeepAlive(resp)
	runtime.KeepAlive(buffer)
}

func newString(s_ref C.StringRef) string {
	return unsafe.String((*byte)(unsafe.Pointer(s_ref.ptr)), s_ref.len)
}
func refString(s *string, _ *[]byte) C.StringRef {
	return C.StringRef{
		ptr: (*C.uint8_t)(unsafe.StringData(*s)),
		len: C.uintptr_t(len(*s)),
	}
}

func cntString(_ *string, _ *uint) [0]C.StringRef { return [0]C.StringRef{} }
func new_list_mapper[T1, T2 any](f func(T1) T2) func(C.ListRef) []T2 {
	return func(x C.ListRef) []T2 {
		input := unsafe.Slice((*T1)(unsafe.Pointer(x.ptr)), x.len)
		output := make([]T2, len(input))
		for i, v := range input {
			output[i] = f(v)
		}
		return output
	}
}
func new_list_mapper_primitive[T1, T2 any](_ func(T1) T2) func(C.ListRef) []T2 {
	return func(x C.ListRef) []T2 {
		return unsafe.Slice((*T2)(unsafe.Pointer(x.ptr)), x.len)
	}
}

// only handle non-primitive type T
func cnt_list_mapper[T, R any](f func(s *T, cnt *uint) [0]R) func(s *[]T, cnt *uint) [0]C.ListRef {
	return func(s *[]T, cnt *uint) [0]C.ListRef {
		for _, v := range *s {
			f(&v, cnt)
		}
		*cnt += uint(len(*s)) * size_of[R]()
		return [0]C.ListRef{}
	}
}

// only handle primitive type T
func cnt_list_mapper_primitive[T, R any](_ func(s *T, cnt *uint) [0]R) func(s *[]T, cnt *uint) [0]C.ListRef {
	return func(s *[]T, cnt *uint) [0]C.ListRef { return [0]C.ListRef{} }
}

// only handle non-primitive type T
func ref_list_mapper[T, R any](f func(s *T, buffer *[]byte) R) func(s *[]T, buffer *[]byte) C.ListRef {
	return func(s *[]T, buffer *[]byte) C.ListRef {
		if len(*buffer) == 0 {
			return C.ListRef{
				ptr: unsafe.Pointer(nil),
				len: C.uintptr_t(len(*s)),
			}
		}
		ret := C.ListRef{
			ptr: unsafe.Pointer(&(*buffer)[0]),
			len: C.uintptr_t(len(*s)),
		}
		children_bytes := int(size_of[R]()) * len(*s)
		children := (*buffer)[:children_bytes]
		*buffer = (*buffer)[children_bytes:]
		for _, v := range *s {
			child := f(&v, buffer)
			len := unsafe.Sizeof(child)
			copy(children, unsafe.Slice((*byte)(unsafe.Pointer(&child)), len))
			children = children[len:]
		}
		return ret
	}
}

// only handle primitive type T
func ref_list_mapper_primitive[T, R any](_ func(s *T, buffer *[]byte) R) func(s *[]T, buffer *[]byte) C.ListRef {
	return func(s *[]T, buffer *[]byte) C.ListRef {
		if len(*s) == 0 {
			return C.ListRef{
				ptr: unsafe.Pointer(nil),
				len: C.uintptr_t(0),
			}
		}
		return C.ListRef{
			ptr: unsafe.Pointer(&(*s)[0]),
			len: C.uintptr_t(len(*s)),
		}
	}
}
func size_of[T any]() uint {
	var t T
	return uint(unsafe.Sizeof(t))
}
func cvt_ref[R, CR any](cnt_f func(s *R, cnt *uint) [0]CR, ref_f func(p *R, buffer *[]byte) CR) func(p *R) (CR, []byte) {
	return func(p *R) (CR, []byte) {
		var cnt uint
		cnt_f(p, &cnt)
		buffer := make([]byte, cnt)
		return ref_f(p, &buffer), buffer
	}
}
func cvt_ref_cap[R, CR any](cnt_f func(s *R, cnt *uint) [0]CR, ref_f func(p *R, buffer *[]byte) CR, add_cap uint) func(p *R) (CR, []byte) {
	return func(p *R) (CR, []byte) {
		var cnt uint
		cnt_f(p, &cnt)
		buffer := make([]byte, cnt, cnt+add_cap)
		return ref_f(p, &buffer), buffer
	}
}

func newC_uint8_t(n C.uint8_t) uint8    { return uint8(n) }
func newC_uint16_t(n C.uint16_t) uint16 { return uint16(n) }
func newC_uint32_t(n C.uint32_t) uint32 { return uint32(n) }
func newC_uint64_t(n C.uint64_t) uint64 { return uint64(n) }
func newC_int8_t(n C.int8_t) int8       { return int8(n) }
func newC_int16_t(n C.int16_t) int16    { return int16(n) }
func newC_int32_t(n C.int32_t) int32    { return int32(n) }
func newC_int64_t(n C.int64_t) int64    { return int64(n) }
func newC_bool(n C.bool) bool           { return bool(n) }
func newC_uintptr_t(n C.uintptr_t) uint { return uint(n) }
func newC_intptr_t(n C.intptr_t) int    { return int(n) }
func newC_float(n C.float) float32      { return float32(n) }
func newC_double(n C.double) float64    { return float64(n) }

func cntC_uint8_t(_ *uint8, _ *uint) [0]C.uint8_t    { return [0]C.uint8_t{} }
func cntC_uint16_t(_ *uint16, _ *uint) [0]C.uint16_t { return [0]C.uint16_t{} }
func cntC_uint32_t(_ *uint32, _ *uint) [0]C.uint32_t { return [0]C.uint32_t{} }
func cntC_uint64_t(_ *uint64, _ *uint) [0]C.uint64_t { return [0]C.uint64_t{} }
func cntC_int8_t(_ *int8, _ *uint) [0]C.int8_t       { return [0]C.int8_t{} }
func cntC_int16_t(_ *int16, _ *uint) [0]C.int16_t    { return [0]C.int16_t{} }
func cntC_int32_t(_ *int32, _ *uint) [0]C.int32_t    { return [0]C.int32_t{} }
func cntC_int64_t(_ *int64, _ *uint) [0]C.int64_t    { return [0]C.int64_t{} }
func cntC_bool(_ *bool, _ *uint) [0]C.bool           { return [0]C.bool{} }
func cntC_uintptr_t(_ *uint, _ *uint) [0]C.uintptr_t { return [0]C.uintptr_t{} }
func cntC_intptr_t(_ *int, _ *uint) [0]C.intptr_t    { return [0]C.intptr_t{} }
func cntC_float(_ *float32, _ *uint) [0]C.float      { return [0]C.float{} }
func cntC_double(_ *float64, _ *uint) [0]C.double    { return [0]C.double{} }

func refC_uint8_t(p *uint8, _ *[]byte) C.uint8_t    { return C.uint8_t(*p) }
func refC_uint16_t(p *uint16, _ *[]byte) C.uint16_t { return C.uint16_t(*p) }
func refC_uint32_t(p *uint32, _ *[]byte) C.uint32_t { return C.uint32_t(*p) }
func refC_uint64_t(p *uint64, _ *[]byte) C.uint64_t { return C.uint64_t(*p) }
func refC_int8_t(p *int8, _ *[]byte) C.int8_t       { return C.int8_t(*p) }
func refC_int16_t(p *int16, _ *[]byte) C.int16_t    { return C.int16_t(*p) }
func refC_int32_t(p *int32, _ *[]byte) C.int32_t    { return C.int32_t(*p) }
func refC_int64_t(p *int64, _ *[]byte) C.int64_t    { return C.int64_t(*p) }
func refC_bool(p *bool, _ *[]byte) C.bool           { return C.bool(*p) }
func refC_uintptr_t(p *uint, _ *[]byte) C.uintptr_t { return C.uintptr_t(*p) }
func refC_intptr_t(p *int, _ *[]byte) C.intptr_t    { return C.intptr_t(*p) }
func refC_float(p *float32, _ *[]byte) C.float      { return C.float(*p) }
func refC_double(p *float64, _ *[]byte) C.double    { return C.double(*p) }

type NetstackRequest struct {
	wg_ip            string
	private_key      string
	public_key       string
	endpoint         string
	dns              string
	ping_hosts       []string
	ping_ips         []string
	num_ping         uint8
	send_timeout_sec uint64
	recv_timeout_sec uint64
	ip_version       uint8
}

func newNetstackRequest(p C.NetstackRequestRef) NetstackRequest {
	return NetstackRequest{
		wg_ip:            newString(p.wg_ip),
		private_key:      newString(p.private_key),
		public_key:       newString(p.public_key),
		endpoint:         newString(p.endpoint),
		dns:              newString(p.dns),
		ping_hosts:       new_list_mapper(newString)(p.ping_hosts),
		ping_ips:         new_list_mapper(newString)(p.ping_ips),
		num_ping:         newC_uint8_t(p.num_ping),
		send_timeout_sec: newC_uint64_t(p.send_timeout_sec),
		recv_timeout_sec: newC_uint64_t(p.recv_timeout_sec),
		ip_version:       newC_uint8_t(p.ip_version),
	}
}
func cntNetstackRequest(s *NetstackRequest, cnt *uint) [0]C.NetstackRequestRef {
	cnt_list_mapper(cntString)(&s.ping_hosts, cnt)
	cnt_list_mapper(cntString)(&s.ping_ips, cnt)
	return [0]C.NetstackRequestRef{}
}
func refNetstackRequest(p *NetstackRequest, buffer *[]byte) C.NetstackRequestRef {
	return C.NetstackRequestRef{
		wg_ip:            refString(&p.wg_ip, buffer),
		private_key:      refString(&p.private_key, buffer),
		public_key:       refString(&p.public_key, buffer),
		endpoint:         refString(&p.endpoint, buffer),
		dns:              refString(&p.dns, buffer),
		ping_hosts:       ref_list_mapper(refString)(&p.ping_hosts, buffer),
		ping_ips:         ref_list_mapper(refString)(&p.ping_ips, buffer),
		num_ping:         refC_uint8_t(&p.num_ping, buffer),
		send_timeout_sec: refC_uint64_t(&p.send_timeout_sec, buffer),
		recv_timeout_sec: refC_uint64_t(&p.recv_timeout_sec, buffer),
		ip_version:       refC_uint8_t(&p.ip_version, buffer),
	}
}

type NetstackResponse struct {
	can_handshake   bool
	sent_ips        uint16
	received_ips    uint16
	sent_hosts      uint16
	received_hosts  uint16
	can_resolve_dns bool
}

func newNetstackResponse(p C.NetstackResponseRef) NetstackResponse {
	return NetstackResponse{
		can_handshake:   newC_bool(p.can_handshake),
		sent_ips:        newC_uint16_t(p.sent_ips),
		received_ips:    newC_uint16_t(p.received_ips),
		sent_hosts:      newC_uint16_t(p.sent_hosts),
		received_hosts:  newC_uint16_t(p.received_hosts),
		can_resolve_dns: newC_bool(p.can_resolve_dns),
	}
}
func cntNetstackResponse(s *NetstackResponse, cnt *uint) [0]C.NetstackResponseRef {
	return [0]C.NetstackResponseRef{}
}
func refNetstackResponse(p *NetstackResponse, buffer *[]byte) C.NetstackResponseRef {
	return C.NetstackResponseRef{
		can_handshake:   refC_bool(&p.can_handshake, buffer),
		sent_ips:        refC_uint16_t(&p.sent_ips, buffer),
		received_ips:    refC_uint16_t(&p.received_ips, buffer),
		sent_hosts:      refC_uint16_t(&p.sent_hosts, buffer),
		received_hosts:  refC_uint16_t(&p.received_hosts, buffer),
		can_resolve_dns: refC_bool(&p.can_resolve_dns, buffer),
	}
}
func main() {}
