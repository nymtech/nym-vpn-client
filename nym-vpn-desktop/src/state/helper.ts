export type TauriReq<
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Req extends (a: never, b?: never) => Promise<any>,
> = {
  name: string;
  request: () => ReturnType<Req>;
  onFulfilled: (value: Awaited<ReturnType<Req>>) => void;
};

// Fires a list of Tauri requests concurrently and handles the results
// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function fireRequests(requests: TauriReq<any>[]) {
  const promises = await Promise.allSettled(requests.map((r) => r.request()));

  promises.forEach((res, index) => {
    if (res.status === 'rejected') {
      console.warn(
        `command [${requests[index].name}] failed with error: ${res.reason}`,
      );
    }
    if (res.status === 'fulfilled') {
      requests[index].onFulfilled(res.value as never);
    }
  });
}

export default fireRequests;
