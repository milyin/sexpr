Решение задачи, описанной в https://swizard.livejournal.com/200401.html

----
Общее условие
Дана программа в виде упрощённого S-expression (https://en.wikipedia.org/wiki/S-expression), например:

sample_1() -> <<"(+ (* 4 4) (* 2 (- 7 5)) 1)">>.


где первый элемент каждого списка — это операция (всего их три вида):

    сложение: атом "+"
    вычитание: атом "-"
    умножение: атом "*"

Все остальные возможные атомы — это целые числа.
Ещё примеры:

sample_2() -> <<"10">>.
sample_3() -> <<"(* 10 (- 0 1))">>.
sample_4() -> <<"(- (+ 10 10) -5 0)">>.
sample_5() -> <<"(+ (- (* (+ (- (* 1))))))">>.
sample_6() -> <<"(* 2 (+ (- 10 9) (- 3 (* 2 1))) (+ (- 10 9) (- 3 (* 2 1))))">>.
sample_7() -> <<"(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))">>.
sample_8() -> <<"(- (+ (+ 3 3) (- 3 3) (+ 3 3) (- 3 3)) (* 2 2))">>.
sample_9() -> <<"(+ (- 6 1) (+ 0 1 1) (- 7 2) (* 3 4 5) (- 3 1) (+ 2) (- 0 10))">>.


Задача 1
Реализуйте функцию-интерпретатор "interpret/1" и вычислите результаты вышеприведённых программ.

Например:

interpret(sample_1()). --> 21
interpret(sample_2()). --> 10
interpret(sample_3()). --> -10
interpret(sample_4()). --> 25
interpret(sample_5()). --> 1
interpret(sample_6()). --> 8
interpret(sample_7()). --> 50
interpret(sample_8()). --> 8
interpret(sample_9()). --> 66


Задача 2
Представим, что все операции в программе имеют константную задержку, которая не зависит от процессора. Например, это некоторого рода длительная сетевая активность.
Итого, реализация каждой из операции теперь должна содержать вызов "timer:sleep(X)":

    X = 2000 ms для сложения "+"
    X = 3000 ms для вычитания "-"
    X = 10000 для умножения "*"


Реализуйте функцию-интерпретатор "interpret_network/1", которая вычисляет результат заданной программы с минимально возможной задержкой.

Например:

interpret_network(sample_1()). --> 21 (delay 15 s)
interpret_network(sample_2()). --> 10 (delay 0 s)
interpret_network(sample_3()). --> -10 (delay 13 s)
interpret_network(sample_4()). --> 25 (delay 5 s)
interpret_network(sample_5()). --> 1 (delay 30 s)
interpret_network(sample_6()). --> 8 (delay 25 s)
interpret_network(sample_7()). --> 50 (delay 15 s)
interpret_network(sample_8()). --> 8 (delay 13 s)
interpret_network(sample_9()). --> 66 (delay 12 s)


Задача 3
Представим, что все операции в программе имеют константную задержку, которая зависит от процессора. Например, это некоторого рода тяжелые вычисления, выедающие 100% ядра процессора.
Наша машина оборудована N физическими ядрами.
Итого, реализация каждой из операции теперь должна содержать вызов "timer:sleep(X)", при этом максимальное количество "одновременно выполняющихся" операций должно быть меньше или равно N:

    X = 2000 ms для сложения "+"
    X = 3000 ms для вычитания "-"
    X = 10000 для умножения "*"


Реализуйте функцию-интерпретатор "interpret_cpu/2", которая, используя процессор максимально оптимальным образом, вычисляет результат заданной программы с минимально возможной задержкой.

Например:

interpret_cpu(sample_1(), 2). --> 21 (delay 15 s)
interpret_cpu(sample_2(), 2). --> 10 (delay 0 s)
interpret_cpu(sample_3(), 2). --> -10 (delay 13 s)
interpret_cpu(sample_4(), 2). --> 25 (delay 5 s)
interpret_cpu(sample_5(), 2). --> 1 (delay 30 s)
interpret_cpu(sample_6(), 2). --> 8 (delay 28 s)
interpret_cpu(sample_6(), 3). --> 8 (delay 25 s)
interpret_cpu(sample_7(), 2). --> 50 (delay 26 s)
interpret_cpu(sample_7(), 3). --> 50 (delay 22 s)
interpret_cpu(sample_7(), 4). --> 50 (delay 15 s)
interpret_cpu(sample_8(), 2). --> 8 (delay 15 s)
interpret_cpu(sample_8(), 3). --> 8 (delay 13 s)
interpret_cpu(sample_9(), 2). --> 66 (delay 15 s)


Логи работы на приведенных примерах:

```
Expression (+ (* 4 4 ) (* 2 (- 7 5 ) ) 1 )
Result 21
Network execution time 15
(* 4 4 ) on cpu 0 start 0 end 10 takes 10
(- 7 5 ) on cpu 1 start 0 end 3 takes 3
(* 2 (- 7 5 ) ) on cpu 1 start 3 end 13 takes 10
(+ (* 4 4 ) (* 2 (- 7 5 ) ) 1 ) on cpu 1 start 13 end 15 takes 2
cpu load [10, 15]
Execution time on 2 cpus is 15 s

Expression 10
Result 10
Network execution time 0
cpu load [0, 0]
Execution time on 2 cpus is 0 s

Expression (* 10 (- 0 1 ) )
Result -10
Network execution time 13
(- 0 1 ) on cpu 0 start 0 end 3 takes 3
(* 10 (- 0 1 ) ) on cpu 0 start 3 end 13 takes 10
cpu load [13, 0]
Execution time on 2 cpus is 13 s

Expression (- (+ 10 10 ) -5 0 )
Result 25
Network execution time 5
(+ 10 10 ) on cpu 0 start 0 end 2 takes 2
(- (+ 10 10 ) -5 0 ) on cpu 0 start 2 end 5 takes 3
cpu load [5, 0]
Execution time on 2 cpus is 5 s

Expression (+ (- (* (+ (- (* 1 ) ) ) ) ) )
Result 1
Network execution time 30
(* 1 ) on cpu 0 start 0 end 10 takes 10
(- (* 1 ) ) on cpu 0 start 10 end 13 takes 3
(+ (- (* 1 ) ) ) on cpu 0 start 13 end 15 takes 2
(* (+ (- (* 1 ) ) ) ) on cpu 0 start 15 end 25 takes 10
(- (* (+ (- (* 1 ) ) ) ) ) on cpu 0 start 25 end 28 takes 3
(+ (- (* (+ (- (* 1 ) ) ) ) ) ) on cpu 0 start 28 end 30 takes 2
cpu load [30, 0]
Execution time on 2 cpus is 30 s

Expression (* 2 (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) )
Result 8
Network execution time 25
(- 10 9 ) on cpu 0 start 0 end 3 takes 3
(* 2 1 ) on cpu 1 start 0 end 10 takes 10
(- 10 9 ) on cpu 0 start 3 end 6 takes 3
(* 2 1 ) on cpu 0 start 6 end 16 takes 10
(- 3 (* 2 1 ) ) on cpu 1 start 10 end 13 takes 3
(+ (- 10 9 ) (- 3 (* 2 1 ) ) ) on cpu 1 start 13 end 15 takes 2
(- 3 (* 2 1 ) ) on cpu 0 start 16 end 19 takes 3
(+ (- 10 9 ) (- 3 (* 2 1 ) ) ) on cpu 0 start 19 end 21 takes 2
(* 2 (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) ) on cpu 0 start 21 end 31 takes 10
cpu load [31, 15]
Execution time on 2 cpus is 31 s

Expression (* 2 (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) )
Result 8
Network execution time 25
(- 10 9 ) on cpu 0 start 0 end 3 takes 3
(* 2 1 ) on cpu 1 start 0 end 10 takes 10
(- 10 9 ) on cpu 2 start 0 end 3 takes 3
(* 2 1 ) on cpu 0 start 3 end 13 takes 10
(- 3 (* 2 1 ) ) on cpu 1 start 10 end 13 takes 3
(+ (- 10 9 ) (- 3 (* 2 1 ) ) ) on cpu 0 start 13 end 15 takes 2
(- 3 (* 2 1 ) ) on cpu 1 start 13 end 16 takes 3
(+ (- 10 9 ) (- 3 (* 2 1 ) ) ) on cpu 1 start 16 end 18 takes 2
(* 2 (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) (+ (- 10 9 ) (- 3 (* 2 1 ) ) ) ) on cpu 1 start 18 end 28 takes 10
cpu load [15, 28, 3]
Execution time on 3 cpus is 28 s

Expression (+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) )
Result 50
Network execution time 15
(* 2 1 ) on cpu 0 start 0 end 10 takes 10
(+ 8 8 ) on cpu 1 start 0 end 2 takes 2
(+ 4 3 2 1 ) on cpu 1 start 2 end 4 takes 2
(* 3 3 ) on cpu 1 start 4 end 14 takes 10
(* 2 2 ) on cpu 0 start 10 end 20 takes 10
(* 5 7 ) on cpu 1 start 14 end 24 takes 10
(- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) on cpu 0 start 20 end 23 takes 3
(+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) ) on cpu 1 start 24 end 26 takes 2
cpu load [23, 26]
Execution time on 2 cpus is 26 s

Expression (+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) )
Result 50
Network execution time 15
(* 2 1 ) on cpu 0 start 0 end 10 takes 10
(+ 8 8 ) on cpu 1 start 0 end 2 takes 2
(+ 4 3 2 1 ) on cpu 2 start 0 end 2 takes 2
(* 3 3 ) on cpu 1 start 2 end 12 takes 10
(* 2 2 ) on cpu 2 start 2 end 12 takes 10
(* 5 7 ) on cpu 0 start 10 end 20 takes 10
(- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) on cpu 1 start 12 end 15 takes 3
(+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) ) on cpu 0 start 20 end 22 takes 2
cpu load [22, 15, 12]
Execution time on 3 cpus is 22 s

Expression (+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) )
Result 50
Network execution time 15
(* 2 1 ) on cpu 0 start 0 end 10 takes 10
(+ 8 8 ) on cpu 1 start 0 end 2 takes 2
(+ 4 3 2 1 ) on cpu 2 start 0 end 2 takes 2
(* 3 3 ) on cpu 3 start 0 end 10 takes 10
(* 2 2 ) on cpu 1 start 2 end 12 takes 10
(* 5 7 ) on cpu 2 start 2 end 12 takes 10
(- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) on cpu 1 start 12 end 15 takes 3
(+ (* 2 1 ) (+ 8 8 ) (- (+ 4 3 2 1 ) (* 3 3 ) (* 2 2 ) ) (* 5 7 ) ) on cpu 1 start 15 end 17 takes 2
cpu load [10, 17, 12, 10]
Execution time on 4 cpus is 17 s

Expression (- (+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) (* 2 2 ) )
Result 8
Network execution time 13
(+ 3 3 ) on cpu 0 start 0 end 2 takes 2
(- 3 3 ) on cpu 1 start 0 end 3 takes 3
(+ 3 3 ) on cpu 0 start 2 end 4 takes 2
(- 3 3 ) on cpu 1 start 3 end 6 takes 3
(* 2 2 ) on cpu 0 start 4 end 14 takes 10
(+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) on cpu 1 start 6 end 8 takes 2
(- (+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) (* 2 2 ) ) on cpu 0 start 14 end 17 takes 3
cpu load [17, 8]
Execution time on 2 cpus is 17 s

Expression (- (+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) (* 2 2 ) )
Result 8
Network execution time 13
(+ 3 3 ) on cpu 0 start 0 end 2 takes 2
(- 3 3 ) on cpu 1 start 0 end 3 takes 3
(+ 3 3 ) on cpu 2 start 0 end 2 takes 2
(- 3 3 ) on cpu 0 start 2 end 5 takes 3
(* 2 2 ) on cpu 2 start 2 end 12 takes 10
(+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) on cpu 0 start 5 end 7 takes 2
(- (+ (+ 3 3 ) (- 3 3 ) (+ 3 3 ) (- 3 3 ) ) (* 2 2 ) ) on cpu 2 start 12 end 15 takes 3
cpu load [7, 3, 15]
Execution time on 3 cpus is 15 s

Expression (+ (- 6 1 ) (+ 0 1 1 ) (- 7 2 ) (* 3 4 5 ) (- 3 1 ) (+ 2 ) (- 0 10 ) )
Result 66
Network execution time 12
(- 6 1 ) on cpu 0 start 0 end 3 takes 3
(+ 0 1 1 ) on cpu 1 start 0 end 2 takes 2
(- 7 2 ) on cpu 1 start 2 end 5 takes 3
(* 3 4 5 ) on cpu 0 start 3 end 13 takes 10
(- 3 1 ) on cpu 1 start 5 end 8 takes 3
(+ 2 ) on cpu 1 start 8 end 10 takes 2
(- 0 10 ) on cpu 1 start 10 end 13 takes 3
(+ (- 6 1 ) (+ 0 1 1 ) (- 7 2 ) (* 3 4 5 ) (- 3 1 ) (+ 2 ) (- 0 10 ) ) on cpu 0 start 13 end 15 takes 2
cpu load [15, 13]
Execution time on 2 cpus is 15 s

```
