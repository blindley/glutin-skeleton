import os
my_directory = os.path.dirname(os.path.realpath(__file__))
output_path = os.path.join(my_directory, "shader.multitext")

def write_vertices(fp):
    vmin = -0.5
    vmax = 0.5
    vertex_positions = [
        [vmin, vmin, vmin],
        [vmin, vmin, vmax],
        [vmin, vmax, vmin],
        [vmin, vmax, vmax],
        [vmax, vmin, vmin],
        [vmax, vmin, vmax],
        [vmax, vmax, vmin],
        [vmax, vmax, vmax]
    ]

    vertex_position_indices = [
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
            pos = vertex_positions[vertex_position_indices[i][j]]
            vertices.extend(pos)

            tx = tx01[tex_order[j][0]] / 768.0
            ty = ty01[tex_order[j][1]] / 512.0
            vertices.extend([tx, ty])

            vertices.extend([1.0, 1.0, 1.0, 1.0])
            if pos[0] < 0:
                vertices.append(0.0)
            else:
                vertices.append(0.5)

    components = [3, 2, 4, 1]
    total_components = sum(components)
    vertex_count = int(len(vertices) / total_components)

    for v in range(vertex_count):
        start = v * total_components
        end = start + total_components
        line = ", ".join(f"{x}" for x in vertices[start:end]) + ","
        fp.write(f"{line}\n")
    
    fp.write("\n")

def write_vertex_components(fp):
    fp.write("3, 2, 4, 1\n\n")

prefix = "@@@"
lines = []
with open(output_path) as fp:
    append = True
    for line in fp.readlines():
        if line.startswith(prefix):
            if "vertices" in line or "vertex components" in line:
                append = False
            else:
                append = True
        if append:
            lines.append(line.rstrip())

with open(output_path, "w+") as fp:
    for line in lines:
        fp.write(f"{line}\n")
    fp.write(f"{prefix} vertex components\n")
    write_vertex_components(fp)
    fp.write(f"{prefix} vertices\n")
    write_vertices(fp)
    
 