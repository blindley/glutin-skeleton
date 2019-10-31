def generate_vertices():
    vmin = -0.5
    vmax = 0.5
    pos = [
        [vmin, vmin, vmin],
        [vmin, vmin, vmax],
        [vmin, vmax, vmin],
        [vmin, vmax, vmax],
        [vmax, vmin, vmin],
        [vmax, vmin, vmax],
        [vmax, vmax, vmin],
        [vmax, vmax, vmax]
    ]

    pos_order = [
        [1, 3, 7, 1, 7, 5],
        [5, 7, 6, 5, 6, 4],
        [4, 6, 2, 4, 2, 0],
        [0, 2, 3, 0, 3, 1],
        [3, 2, 6, 3, 6, 7],
        [0, 1, 5, 0, 5, 4]
    ]

    tex_order = [
        [0, 1], [0, 0], [1, 0], [0, 1], [1, 0], [1, 1]
    ]

    vertices = []
    for i in range(6):
        tx0 = ((i % 3) * 256.0)
        ty0 = (int(i / 3) * 256.0)
        tx01 = [tx0 + 0.5, tx0 + 255.5]
        ty01 = [ty0 + 0.5, ty0 + 255.5]
        for j in range(6):
            vertices.extend(pos[pos_order[i][j]])

            tx = tx01[tex_order[j][0]] / 768.0
            ty = ty01[tex_order[j][1]] / 512.0

            if tx > 1.0 or ty > 1.0:
                print(f"i={i}, j={j}, tx={tx}, ty={ty}")

            vertices.extend([tx, ty])

            # red
            vertices.extend([1.0, 0.0, 0.0, 1.0])

            # texture/color mix ratio
            vertices.append(0.75)

    components = [3, 2, 4, 1]
    total_components = sum(components)
    vertex_count = int(len(vertices) / total_components)
    
    for v in range(vertex_count):
        start = v * total_components
        end = start + total_components
        print(", ".join(f"{x}" for x in vertices[start:end]) + ",")

    print(f"components: {components}")
    print(f"vertex count: {vertex_count}")

generate_vertices()
