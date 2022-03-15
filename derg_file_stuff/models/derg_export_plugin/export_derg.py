import os

import bpy
from mathutils import Matrix, Vector, Color
from bpy_extras import io_utils, node_shader_utils

from bpy_extras.wm_utils.progress_report import (
    ProgressReport,
    ProgressReportSubstep,
)

def name_compat(name):
    if name is None:
        return 'None'
    else:
        return name.replace(' ', '_')

def mesh_triangulate(me):
    import bmesh
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.triangulate(bm, faces=bm.faces)
    bm.to_mesh(me)
    bm.free()

def write_file(filepath, objects, depsgraph, scene,
               EXPORT_APPLY_MODIFIERS=True,
               EXPORT_APPLY_MODIFIERS_RENDER=False,
               EXPORT_BLEN_OBS=True,
               EXPORT_GROUP_BY_OB=False,
               EXPORT_GLOBAL_MATRIX=None,
               progress=ProgressReport(),
               ):
    if EXPORT_GLOBAL_MATRIX is None:
        EXPORT_GLOBAL_MATRIX = Matrix()

    def veckey3d(v):
        return round(v.x, 4), round(v.y, 4), round(v.z, 4)

    def veckey2d(v):
        return round(v[0], 4), round(v[1], 4)

    def findVertexGroupName(face, vWeightMap):
        weightDict = {}
        for vert_index in face.vertices:
            vWeights = vWeightMap[vert_index]
            for vGroupName, weight in vWeights:
                weightDict[vGroupName] = weightDict.get(vGroupName, 0.0) + weight

        if weightDict:
            return max((weight, vGroupName) for vGroupName, weight in weightDict.items())[1]
        else:
            return '(null)'

    with ProgressReportSubstep(progress, 2, "DERG Export path: %r" % filepath, "DERG Export Finished") as subprogress1:
        with open(filepath, "wb") as f:
            fw = f.write
            copy_set = set()

            # Get all meshes
            subprogress1.enter_substeps(len(objects))

            # Write object length
            fw(bytes(len(objects)))
"""
            for i, ob_main in enumerate(objects):
                # ignore dupli children
                if ob_main.parent and ob_main.parent.instance_type in {'VERTS', 'FACES'}:
                    subprogress1.step("Ignoring %s, dupli child..." % ob_main.name)
                    continue

                obs = [(ob_main, ob_main.matrix_world)]
                if ob_main.is_instancer:
                    obs += [(dup.instance_object.original, dup.matrix_world.copy())
                            for dup in depsgraph.object_instances
                            if dup.parent and dup.parent.original == ob_main]

                subprogress1.enter_substeps(len(obs))
                for ob, ob_mat in obs:
                    with ProgressReportSubstep(subprogress1, 6) as subprogress2:
                        uv_unique_count = no_unique_count = 0
                        
                        ob_for_convert = ob.evaluated_get(depsgraph) if EXPORT_APPLY_MODIFIERS else ob.original

                        try:
                            me = ob_for_convert.to_mesh()
                        except RuntimeError:
                            me = None

                        if me is None:
                            continue

                        # _must_ do this before applying transformation, else tessellation may differ
                        # _must_ do this first since it re-allocs arrays
                        mesh_triangulate(me)

                        me.transform(EXPORT_GLOBAL_MATRIX @ ob_mat)
                        # If negative scaling, we have to invert the normals...
                        if ob_mat.determinant() < 0.0:
                            me.flip_normals()


                        faceuv = len(me.uv_layers) > 0
                        if faceuv:
                            uv_layer = me.uv_layers.active.data[:]

                        me_verts = me.vertices[:]

                        # Make our own list so it can be sorted to reduce context switching
                        face_index_pairs = [(face, index) for index, face in enumerate(me.polygons)]

                        if not (len(face_index_pairs) + len(me.vertices)):  # Make sure there is something to write
                            # clean up
                            ob_for_convert.to_mesh_clear()
                            continue  # dont bother with this mesh.

                        me.calc_normals_split()
                        # No need to call me.free_normals_split later, as this mesh is deleted anyway!

                        loops = me.loops

                        if EXPORT_BLEN_OBS or EXPORT_GROUP_BY_OB:
                            name1 = ob.name
                            name2 = ob.data.name
                            if name1 == name2:
                                obnamestring = name_compat(name1)
                            else:
                                obnamestring = '%s_%s' % (name_compat(name1), name_compat(name2))

                            if EXPORT_BLEN_OBS:
                                fw('o %s\n' % obnamestring)  # Write Object name
                            else:  # if EXPORT_GROUP_BY_OB:
                                fw('g %s\n' % obnamestring)

                        subprogress2.step()

                        # Vert
                        for v in me_verts:
                            fw('v %.6f %.6f %.6f\n' % v.co[:])

                        subprogress2.step()

                        # UV
                        # in case removing some of these dont get defined.
                        uv = f_index = uv_index = uv_key = uv_val = uv_ls = None

                        uv_face_mapping = [None] * len(face_index_pairs)

                        uv_dict = {}
                        uv_get = uv_dict.get
                        for f, f_index in face_index_pairs:
                            uv_ls = uv_face_mapping[f_index] = []
                            for uv_index, l_index in enumerate(f.loop_indices):
                                uv = uv_layer[l_index].uv
                                # include the vertex index in the key so we don't share UV's between vertices,
                                # allowed by the OBJ spec but can cause issues for other importers, see: T47010.

                                # this works too, shared UV's for all verts
                                #~ uv_key = veckey2d(uv)
                                uv_key = loops[l_index].vertex_index, veckey2d(uv)

                                uv_val = uv_get(uv_key)
                                if uv_val is None:
                                    uv_val = uv_dict[uv_key] = uv_unique_count
                                    fw('vt %.6f %.6f\n' % uv[:])
                                    uv_unique_count += 1
                                uv_ls.append(uv_val)

                        del uv_dict, uv, f_index, uv_index, uv_ls, uv_get, uv_key, uv_val
                        # Only need uv_unique_count and uv_face_mapping

                        subprogress2.step()

                        # NORMAL, Smooth/Non smoothed.
                        no_key = no_val = None
                        normals_to_idx = {}
                        no_get = normals_to_idx.get
                        loops_to_normals = [0] * len(loops)
                        for f, f_index in face_index_pairs:
                            for l_idx in f.loop_indices:
                                no_key = veckey3d(loops[l_idx].normal)
                                no_val = no_get(no_key)
                                if no_val is None:
                                    no_val = normals_to_idx[no_key] = no_unique_count
                                    fw('vn %.4f %.4f %.4f\n' % no_key)
                                    no_unique_count += 1
                                loops_to_normals[l_idx] = no_val
                        del normals_to_idx, no_get, no_key, no_val

                        subprogress2.step()

                        # Write the vertex group
                        vertGroupNames = ob.vertex_groups.keys()
                        if vertGroupNames:
                            currentVGroup = ''
                            # Create a dictionary keyed by face id and listing, for each vertex, the vertex groups it belongs to
                            vgroupsMap = [[] for _i in range(len(me_verts))]
                            # find what vertext group the face belongs to
                            vgroup_of_face = findVertexGroupName(f, vgroupsMap)
                            if vgroup_of_face != currentVGroup:
                                currentVGroup = vgroup_of_face
                                fw('g %s\n' % vgroup_of_face)

                        f_v = [(vi, me_verts[v_idx], l_idx)
                                for vi, (v_idx, l_idx) in enumerate(zip(f.vertices, f.loop_indices))]

                        fw('f')
                        faceuv = len(me.uv_layers) > 0
                        if faceuv:
                            for vi, v, li in f_v:
                                fw(" %d/%d/%d" % (totverts + v.index,
                                                    totuvco + uv_face_mapping[f_index][vi],
                                                    totno + loops_to_normals[li],
                                                    ))  # vert, uv, normal

                            face_vert_index += len(f_v)

                        else:  # No UV's
                            if EXPORT_NORMALS:
                                for vi, v, li in f_v:
                                    fw(" %d//%d" % (totverts + v.index, totno + loops_to_normals[li]))
                            else:  # No Normals
                                for vi, v, li in f_v:
                                    fw(" %d" % (totverts + v.index))

                        fw('\n')

                        subprogress2.step()

                        # Make the indices global rather then per mesh
                        totverts += len(me_verts)
                        totuvco += uv_unique_count
                        totno += no_unique_count

                        # clean up
                        ob_for_convert.to_mesh_clear()

                subprogress1.leave_substeps("Finished writing geometry of '%s'." % ob_main.name)
            subprogress1.leave_substeps()
"""

            # copy all collected files.
            io_utils.path_reference_copy(copy_set)

def _write(context, filepath,
           EXPORT_APPLY_MODIFIERS,  # ok
           EXPORT_APPLY_MODIFIERS_RENDER,  # ok
           EXPORT_BLEN_OBS,
           EXPORT_GROUP_BY_OB,
           EXPORT_SEL_ONLY,  # ok
           EXPORT_ANIMATION,
           EXPORT_GLOBAL_MATRIX,
           ):

    with ProgressReport(context.window_manager) as progress:
        base_name, ext = os.path.splitext(filepath)
        context_name = [base_name, '', '', ext]  # Base name, scene name, frame number, extension

        depsgraph = context.evaluated_depsgraph_get()
        scene = context.scene

        # Exit edit mode before exporting, so current object states are exported properly.
        if bpy.ops.object.mode_set.poll():
            bpy.ops.object.mode_set(mode='OBJECT')

        orig_frame = scene.frame_current

        # Export an animation?
        if EXPORT_ANIMATION:
            scene_frames = range(scene.frame_start, scene.frame_end + 1)  # Up to and including the end frame.
        else:
            scene_frames = [orig_frame]  # Dont export an animation.

        # Loop through all frames in the scene and export.
        progress.enter_substeps(len(scene_frames))
        for frame in scene_frames:
            if EXPORT_ANIMATION:  # Add frame to the filepath.
                context_name[2] = '_%.6d' % frame

            scene.frame_set(frame, subframe=0.0)
            if EXPORT_SEL_ONLY:
                objects = context.selected_objects
            else:
                objects = scene.objects

            full_path = ''.join(context_name)

            # erm... bit of a problem here, this can overwrite files when exporting frames. not too bad.
            # EXPORT THE FILE.
            progress.enter_substeps(1)
            write_file(full_path, objects, depsgraph, scene,
                       EXPORT_APPLY_MODIFIERS,
                       EXPORT_APPLY_MODIFIERS_RENDER,
                       EXPORT_BLEN_OBS,
                       EXPORT_GROUP_BY_OB,
                       EXPORT_GLOBAL_MATRIX,
                       progress,
                       )
            progress.leave_substeps()

        scene.frame_set(orig_frame, subframe=0.0)
        progress.leave_substeps()

def save(context,
         filepath,
         *,
         use_mesh_modifiers=True,
         use_mesh_modifiers_render=False,
         use_blen_objects=True,
         group_by_object=False,
         use_vertex_groups=False,
         use_selection=True,
         use_animation=False,
         global_matrix=None,
         path_mode='AUTO'
         ):

    _write(context, filepath,
           EXPORT_APPLY_MODIFIERS=use_mesh_modifiers,
           EXPORT_APPLY_MODIFIERS_RENDER=use_mesh_modifiers_render,
           EXPORT_BLEN_OBS=use_blen_objects,
           EXPORT_GROUP_BY_OB=group_by_object,
           EXPORT_SEL_ONLY=use_selection,
           EXPORT_ANIMATION=use_animation,
           EXPORT_GLOBAL_MATRIX=global_matrix,
           )

    return {'FINISHED'}
