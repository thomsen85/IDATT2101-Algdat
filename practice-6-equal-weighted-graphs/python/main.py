from collections import deque
import re

def load_graph_from_file(path):
    points = []
    length = None
    first_time = True
    with open(path, "r") as f:
        lines = f.readlines() 
        for line in lines:
            formatted_line = re.sub(r"\s+", " ", line)
            points.append(list(map(lambda x: int(x), formatted_line.strip().split(" "))))

            if first_time:
                length, _ = points.pop()
                first_time = False

    graph = []
    for _ in range(length):
        graph.append([])

    for (f, t) in points:
        graph[f].append(t)
    return graph


class TarjanSolver:
    """ Implementation of Tarjan's strongly connected components algorithm"""

    UNVISITED = -1

    def __init__(self, graph):
        self.graph = graph
        self.n = len(graph)

        self.solved = False
        self.scc_count = 0

        self.id = 0
        self.ids = [self.UNVISITED] * self.n
        self.low = [0] * self.n
        self.sccs = [0] * self.n
        self.visited = [False] * self.n

        self.stack = deque()

    def get_scc_count(self):
        if not self.solved:
            self.solve()
        return self.scc_count

    def get_sccs(self):
        ans = []
        print(self.sccs)
        if not self.solved:
            self.solve()

        for _ in range(max(self.sccs)+1):
            ans.append([])
        for (i, group) in enumerate(self.sccs):
            ans[group].append(i)

        return ans

    def solve(self):
        if self.solved:
            return

        for i in range(self.n):
            if self.ids[i] == self.UNVISITED:
                self.dfs(i)

        self.solved = True

    def dfs(self, at):
        self.id += 1
        self.low[at] = self.id
        self.ids[at] = self.id

        self.stack.appendleft(at)
        self.visited[at] = True

        for to in self.graph[at]:
            if self.ids[to] == self.UNVISITED:
                self.dfs(to)

            if self.visited[to]:
                self.low[at] = min(self.low[at], self.low[to])

        if self.ids[at] == self.low[at]:
            while len(self.stack) > 0:
                node = self.stack.popleft()
                self.visited[node] = False
                self.sccs[node] = self.scc_count
                if node == at:
                    break
            self.scc_count += 1


graph_paths = ["ø6g1.txt", "ø6g2.txt", "ø6g5.txt", "ø6g6.txt"]
for path in graph_paths:
    graph = load_graph_from_file(path)
    solver = TarjanSolver(graph)
    print(f"Found {solver.get_scc_count()} SCCs for graph {path}:")
    if solver.get_scc_count() < 100:
        for group in solver.get_sccs():
            if len(group) > 0:
                for num in group:
                    print(num, end=" ")
                print()
