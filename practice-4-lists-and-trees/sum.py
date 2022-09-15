import sys

class Node:
    def __init__(self, prev, next, data: int):
        self.prev = prev
        self.next = next
        self.data = data
    
    def __str__(self) -> str:
        if self.next == None:
            return str(self.data)
        else:
            return str(self.data) + " - " + str(self.next)

    def get_last(root):
        while root.next != None:
            root = root.next

        return root
    
    def __len__(self) -> int:
        counter = 1
        while self.next != None:
            self = self.next
            counter += 1

        return counter

    def normalized(self, length):
        for _ in range(length - len(self)):
            self.prev = Node(None, self, 0)
            self = self.prev
        return self 

    def __gt__(self, other):
        if len(self) != len(other):
            max_len = max(len(self), len(other))

            self = self.normalized(max_len)
            other = other.normalized(max_len)

        while self.next is not None:
            if self.data == other.data:
                self = self.next
                other = other.next
            elif self.data > other.data:
                return True
            elif self.data < other.data:
                return False

        return False


    def __sub__(self, other):
        max_len = max(len(self), len(other))

        self = self.normalized(max_len)
        other = other.normalized(max_len)

        borrowing = True
        ans = []
        negative = False

        if other > self:
            negative = True
            self, other = other, self

        self = self.get_last()
        other = other.get_last()

        while self is not None:
            if self.data < other.data:
                root = self
                self = self.prev
            
                while self.data < 1:
                    self.data += 9
                    print(self.data, "is less than one")
                    self = self.prev
                    
                self.data -= 1
                self = root
                self.data += 10
                
                
            buff = self.data - other.data
            if buff != 0:
                ans.append(str(buff))
            
            self = self.prev
            other = other.prev

        if negative: 
            ans.append("-")
        if len(ans) == 0:
            ans.append("0")

        return "".join(ans[::-1])
            

    def __add__(self, other):
        max_len = max(len(self), len(other))

        self = self.normalized(max_len)
        other = other.normalized(max_len)

        self = self.get_last()
        other = other.get_last()

        overflow = False
        ans = [] 
        while True:
            buff = self.data + other.data

            if overflow:
                buff += 1
                overflow = False
            if buff >= 10:
                overflow = True
                buff %= 10


            ans.append(buff)

            if self.prev is None or other.prev is None:
                break

            self = self.prev
            other = other.prev
            
        if overflow:
            ans.append(1)

        return "".join(map(lambda i: str(i), ans[::-1]))



if len(sys.argv) != 4:
    print("Please use command line arguments, python <program> <num> <operator> <num>")
    exit()

(_, num1, operator, num2) = sys.argv

l_spaces = max(len(num1), len(num2)) + 2
print(f"{num1.rjust(l_spaces)}\n{operator} {num2.rjust(l_spaces-2)}")

num1 = list(num1)
num2 = list(num2)

root_1 = Node(None, None, int(num1[0]))
pre_1 = root_1
for i in range(1, len(num1)):
    i_node = Node(pre_1, None, int(num1[i]))
    pre_1.next = i_node
    pre_1 = i_node

root_2 = Node(None, None, int(num2[0]))
pre_2 = root_2
for i in range(1, len(num2)):
    i_node = Node(pre_2, None, int(num2[i]))
    pre_2.next = i_node
    pre_2 = i_node

if operator == "+":
    print("=", root_1 + root_2)
elif operator == "-":
    print("=", root_1 - root_2)
else:
    print("Unknowned operator please use + or -")
        
