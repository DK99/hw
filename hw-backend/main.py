import asyncio
from grpclib.client import Channel
from models.platform import PlatformStub


async def handle_profiles():
    channel: Channel = Channel(host="api.hwctf.alles.team", port=443)
    platform: PlatformStub = PlatformStub(channel)

    async for response in platform.get_profiles(streaming=True):
        print(response)

    channel.close()


async def main():
    asyncio.create_task(handle_profiles())

    this_task = asyncio.current_task()
    all_tasks = [
        task for task in asyncio.all_tasks()
        if task is not this_task]
    await asyncio.wait(all_tasks)

if __name__ == "__main__":
    asyncio.run(main())
