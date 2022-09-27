import sys
import turtle

class Node:
    def __init__(self, left, right, data):
        self.left = left
        self.right = right
        self.data = data

if len(sys.argv) <= 1:
    print("Please write words as arguments")
    exit()

root = Node(None, None, sys.argv[1])
for word in sys.argv[2:]:
    current_node = root
    while True:
        if current_node.data > word:
            if current_node.left is None:
                current_node.left = Node(None, None, word)
                break
            else:
                current_node = current_node.left
        else:
            if current_node.right is None:
                current_node.right = Node(None, None, word)
                break
            else:
                current_node = current_node.right

style = ('Arial', 16, 'italic')
def move_to(pos):
    turtle.penup()
    turtle.setpos(pos)
    turtle.pendown()

def recursion_draw(n, node, is_left):
    if n < 4 and node is not None:

        if is_left:
            turtle.setheading(270 - (75 // (n+1)))
            turtle.forward(140)
        else:
            turtle.setheading(270 + (75 // (n+1)))
            turtle.forward(140)


        turtle.color("red")
        turtle.write(node.data, font=style, align='center')
        turtle.color("black")
        pos = turtle.pos()
        
        recursion_draw(n+1, node.left, True)
        move_to(pos)
        recursion_draw(n+1, node.right, False)
        move_to(pos)

turtle.speed(0)
turtle.hideturtle()
move_to((0, 200))
turtle.write(root.data, font=style, align='center')
recursion_draw(0, root.left, True)
move_to((0, 200))
recursion_draw(0, root.right, False)
turtle.done()

