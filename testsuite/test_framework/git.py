# A wrapper around git operations

from dataclasses import dataclass
from typing import Generator
from .shell import Shell, RunResult


@dataclass
class Git:
    shell: Shell

    def run(self, command) -> RunResult:
        return self.shell.run(["git", *command])

    def last(self, limit: int = 1) -> Generator[str, None, None]:
        for i in range(limit):
            yield self.run(["rev-parse", f"HEAD~{i}"]).unwrap().decode().strip()

    def branch(self) -> str:
        return self.run(["rev-parse", "--abbrev-ref", "HEAD"]).unwrap().decode().strip()

    def is_clean(self) -> bool:
        """Check if the git working directory is clean, using git status --porcelain"""
        ret = self.run(["status", "--porcelain"])
        succ = ret.succeeded()
        out = ret.unwrap().decode().strip()
        out_empty = len(out) == 0
        return succ and out_empty

    def remote_matches(self, remote: str, ref: str) -> bool:
        """Check if the current branch matches the remote branch"""
        # git ls-remote --heads  https://github.com/aptos-labs/aptos-core.git rustielin/exp
        remote_commit_hash = self.get_remote_commit_hash(remote, ref)
        local_commit_hash = self.get_commit_hash(ref)
        return remote_commit_hash == local_commit_hash

    def get_remote_commit_hash(self, remote: str, ref: str) -> str:
        try:
            return (
                self.run(["ls-remote", "--heads", remote, ref])
                .unwrap()
                .decode()
                .strip()
                .split()[0]
            )
        except IndexError as e:
            raise Exception(f"Remote {remote} does not have branch {ref}: {e}")
        except Exception as e:
            raise Exception(f"Error fetching remote {remote} branch {ref}: {e}")

    def get_commit_hash(self, ref: str) -> str:
        return self.run(["rev-parse", ref]).unwrap().decode().strip()
