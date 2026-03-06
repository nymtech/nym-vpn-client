#include <linux/netlink.h>
#include <linux/connector.h>
#include <linux/cn_proc.h>

struct __attribute__((__packed__)) nlcn_subscribe_payload
{
    struct cn_msg cn_msg;
    enum proc_cn_mcast_op cn_mcast;
} inner;

struct __attribute__((aligned(NLMSG_ALIGNTO))) nlcn_subscribe_msg
{
    struct nlmsghdr nl_hdr;
    struct nlcn_subscribe_payload payload;
};

struct __attribute__((__packed__)) nlcn_event_payload
{
    struct cn_msg cn_msg;
    struct proc_event proc_ev;
};

struct __attribute__((aligned(NLMSG_ALIGNTO))) nlcn_event_msg
{
    struct nlmsghdr nl_hdr;
    struct nlcn_event_payload payload;
};
