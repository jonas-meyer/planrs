/// Generate a [`ProjectRow`] impl.
///
/// ```ignore
/// impl_project_row!(CityModel, NetworkNode, NetworkNodeId, nodes {
///     type Extra = NodeWorldPosition;
///     fn project(_id, node, _model) {
///         Some(NodeWorldPosition(node.position))
///     }
/// });
/// ```
///
/// For cross-projection lookups, use the 4-arg form to access `&World`:
///
/// ```ignore
/// impl_project_row!(CityModel, Attachment, AttachmentId, attachments {
///     type Extra = (AttachmentWorldPosition, ChildOf);
///     fn project(_id, att, model, world) {
///         let edge_proj = world.resource::<RowProjection<EdgeId>>();
///         // ...
///     }
/// });
/// ```
#[macro_export]
macro_rules! impl_project_row {
    (
        $model:ty, $row:ty, $id:ty, $($path:ident).+ {
            type Extra = $extra:ty;
            fn project($id_var:ident, $row_var:ident, $model_var:ident, $world_var:ident)
                $body:block
        }
    ) => {
        impl $crate::ProjectRow<$model> for $row {
            type Id = $id;
            type Extra = $extra;

            fn store(model: &$model) -> &$crate::Store<$id, $row> {
                &model.$($path).+
            }

            fn store_mut(model: &mut $model) -> &mut $crate::Store<$id, $row> {
                &mut model.$($path).+
            }

            fn project(
                $id_var: $id,
                $row_var: &$row,
                $model_var: &$model,
                $world_var: &$crate::World,
            ) -> Option<$extra> $body
        }
    };

    // Without world access
    (
        $model:ty, $row:ty, $id:ty, $($path:ident).+ {
            type Extra = $extra:ty;
            fn project($id_var:ident, $row_var:ident, $model_var:ident) $body:block
        }
    ) => {
        $crate::impl_project_row! {
            $model, $row, $id, $($path).+ {
                type Extra = $extra;
                fn project($id_var, $row_var, $model_var, _world) $body
            }
        }
    };
}
