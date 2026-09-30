def putin():
    return map(int, input().split())


def sol():
    n = int(input())
    C = list(putin())
    B = list(putin())
    q = int(input())
    x = int(input())
    min_arr = [x]
    min_part_sums = [x]
    part_sums = [C[0]]
    for i in range(1, n):
        part_sums.append(part_sums[-1] + C[i])
    for elem in B:
        min_arr.append(min_arr[-1] + elem)
        min_part_sums.append(min_arr[-1] + min_part_sums[-1])
    for i in range(n):
        if min_part_sums[i] > part_sums[i]:
            return 0
    if min_part_sums[0] > C[0]:
        return 0
    answer = [1] * (part_sums[0] - max(0, min_part_sums[0]) + 1)
    for k in range(1, n):
        new_answer = [0] * (part_sums[k] - max(0, min_part_sums[k]) + 1)
        cnt = 1
        window = answer[-1]
        new_answer[-1] = window
        while cnt <= len(new_answer) - 1:
            cnt += 1
            if cnt <= len(answer):
                window += answer[-cnt]
            if C[k] + 1 < cnt:
                window -= answer[C[k] + 1 - cnt]
            new_answer[-cnt] = window
        answer = new_answer.copy()
    m = 10 ** 9 + 7
    return sum(answer) % m


print(sol())